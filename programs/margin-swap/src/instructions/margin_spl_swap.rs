// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright (C) 2022 JET PROTOCOL HOLDINGS, LLC.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use anchor_spl::token::Token;
use jet_margin_pool::ChangeKind;
use jet_static_program_registry::{
    orca_swap_v1, orca_swap_v2, related_programs, spl_token_swap_v2,
};

use crate::*;

// register permitted swap programs
related_programs! {
    SwapProgram {[
        spl_token_swap_v2::Spl2,
        orca_swap_v1::OrcaV1,
        orca_swap_v2::OrcaV2,
    ]}
}

#[derive(Accounts)]
pub struct MarginSplSwap<'info> {
    /// The margin account being executed on
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The account with the source deposit to be exchanged from
    /// CHECK:
    #[account(mut)]
    pub source_account: AccountInfo<'info>,

    /// The destination account to send the deposit that is exchanged into
    /// CHECK:
    #[account(mut)]
    pub destination_account: AccountInfo<'info>,

    /// Temporary account for moving tokens
    /// CHECK:
    #[account(mut)]
    pub transit_source_account: AccountInfo<'info>,

    /// Temporary account for moving tokens
    /// CHECK:
    #[account(mut)]
    pub transit_destination_account: AccountInfo<'info>,

    /// The accounts relevant to the swap pool used for the exchange
    pub swap_info: SwapInfo<'info>,

    /// The accounts relevant to the source margin pool
    pub source_margin_pool: MarginPoolInfo<'info>,

    /// The accounts relevant to the destination margin pool
    pub destination_margin_pool: MarginPoolInfo<'info>,

    pub margin_pool_program: Program<'info, JetMarginPool>,

    pub token_program: Program<'info, Token>,
}

impl<'info> MarginSplSwap<'info> {
    #[inline(never)]
    fn withdraw(&self, change_kind: ChangeKind, amount_in: u64) -> Result<()> {
        jet_margin_pool::cpi::withdraw(
            CpiContext::new(
                self.margin_pool_program.to_account_info(),
                Withdraw {
                    margin_pool: self.source_margin_pool.margin_pool.to_account_info(),
                    vault: self.source_margin_pool.vault.to_account_info(),
                    deposit_note_mint: self.source_margin_pool.deposit_note_mint.to_account_info(),
                    depositor: self.margin_account.to_account_info(),
                    source: self.source_account.to_account_info(),
                    destination: self.transit_source_account.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                },
            ),
            change_kind,
            amount_in,
        )?;

        Ok(())
    }

    #[inline(never)]
    fn deposit(&self, destination_amount: u64) -> Result<()> {
        jet_margin_pool::cpi::deposit(
            CpiContext::new(
                self.margin_pool_program.to_account_info(),
                Deposit {
                    margin_pool: self.destination_margin_pool.margin_pool.to_account_info(),
                    vault: self.destination_margin_pool.vault.to_account_info(),
                    deposit_note_mint: self
                        .destination_margin_pool
                        .deposit_note_mint
                        .to_account_info(),
                    depositor: self.margin_account.to_account_info(),
                    source: self.transit_destination_account.to_account_info(),
                    destination: self.destination_account.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                },
            ),
            ChangeKind::ShiftBy,
            destination_amount,
        )?;

        Ok(())
    }

    #[inline(never)]
    fn swap(&self, amount_in: u64, minimum_amount_out: u64) -> Result<()> {
        let swap_ix = use_client!(self.swap_info.swap_program.key(), {
            client::instruction::swap(
                self.swap_info.swap_program.key,
                self.token_program.key,
                self.swap_info.swap_pool.key,
                self.swap_info.authority.key,
                &self.margin_account.key(),
                &self.transit_source_account.key(),
                self.swap_info.vault_into.key,
                self.swap_info.vault_from.key,
                &self.transit_destination_account.key(),
                self.swap_info.token_mint.key,
                self.swap_info.fee_account.key,
                None,
                client::instruction::Swap {
                    amount_in,
                    minimum_amount_out,
                },
            )?
        })?;

        invoke(
            &swap_ix,
            &[
                self.swap_info.swap_pool.to_account_info(),
                self.margin_account.to_account_info(),
                self.swap_info.authority.to_account_info(),
                self.transit_source_account.to_account_info(),
                self.swap_info.vault_into.to_account_info(),
                self.swap_info.vault_from.to_account_info(),
                self.transit_destination_account.to_account_info(),
                self.swap_info.token_mint.to_account_info(),
                self.swap_info.fee_account.to_account_info(),
                self.token_program.to_account_info(),
            ],
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct SwapInfo<'info> {
    /// CHECK:
    pub swap_pool: UncheckedAccount<'info>,

    /// CHECK:
    pub authority: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub vault_into: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub vault_from: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub token_mint: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub fee_account: UncheckedAccount<'info>,

    /// The address of the swap program
    /// CHECK:
    pub swap_program: UncheckedAccount<'info>,
}

/// Execute a swap by withdrawing tokens from a deposit pool, swapping them for
/// other tokens, then depositing those other tokens to another deposit pool.
///
/// The instruction uses 'transit' accounts which are normally ATAs owned by the
/// margin account. To ensure that only the tokens withdrawn are swapped and
/// deposited, the instruction checks the balances of the transit accounts before
/// and after an action.
/// If either transit account has tokens before the instructions, it should still
/// have the same tokens after the swap.
pub fn margin_spl_swap_handler(
    ctx: Context<MarginSplSwap>,
    withdrawal_change_kind: ChangeKind,
    withdrawal_amount: u64,
    minimum_amount_out: u64,
) -> Result<()> {
    // Get the balance before the withdrawal. The balance should almost always
    // be zero, however it could already have a value.
    let source_opening_balance =
        token::accessor::amount(&ctx.accounts.transit_source_account.to_account_info())?;
    ctx.accounts
        .withdraw(withdrawal_change_kind, withdrawal_amount)?;
    let source_closing_balance =
        token::accessor::amount(&ctx.accounts.transit_source_account.to_account_info())?;

    // The closing balance should be > opening balance after the withdrawal
    let swap_amount_in = source_closing_balance
        .checked_sub(source_opening_balance)
        .unwrap();
    if swap_amount_in == 0 {
        return err!(crate::ErrorCode::NoSwapTokensWithdrawn);
    }

    let destination_opening_balance =
        token::accessor::amount(&ctx.accounts.transit_destination_account.to_account_info())?;
    ctx.accounts.swap(swap_amount_in, minimum_amount_out)?;
    let destination_closing_balance =
        token::accessor::amount(&ctx.accounts.transit_destination_account.to_account_info())?;

    // If the swap would have resulted in 0 tokens, the swap program would error out,
    // thus balance below will be positive.
    let swap_amount_out = destination_closing_balance
        .checked_sub(destination_opening_balance)
        .unwrap();
    ctx.accounts.deposit(swap_amount_out)?;

    Ok(())
}
