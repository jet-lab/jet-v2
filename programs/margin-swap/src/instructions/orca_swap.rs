// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright (C) 2023 JET PROTOCOL HOLDINGS, LLC.
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
use orca_whirlpool::program::Whirlpool;

use crate::*;

#[derive(Accounts)]
pub struct OrcaWhirlpoolSwapPool<'info> {
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
    pub swap_info: WhirlpoolSwapInfo<'info>,

    /// The accounts relevant to the source margin pool
    pub source_margin_pool: MarginPoolInfo<'info>,

    /// The accounts relevant to the destination margin pool
    pub destination_margin_pool: MarginPoolInfo<'info>,

    pub margin_pool_program: Program<'info, JetMarginPool>,

    pub token_program: Program<'info, Token>,

    /// The address of the swap program
    pub swap_program: Program<'info, Whirlpool>,
}

impl<'info> OrcaWhirlpoolSwapPool<'info> {
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
    fn deposit_destination(&self, amount: u64) -> Result<()> {
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
            amount,
        )?;

        Ok(())
    }

    #[inline(never)]
    fn deposit_source(&self, amount: u64) -> Result<()> {
        jet_margin_pool::cpi::deposit(
            CpiContext::new(
                self.margin_pool_program.to_account_info(),
                Deposit {
                    margin_pool: self.source_margin_pool.margin_pool.to_account_info(),
                    vault: self.source_margin_pool.vault.to_account_info(),
                    deposit_note_mint: self.source_margin_pool.deposit_note_mint.to_account_info(),
                    depositor: self.margin_account.to_account_info(),
                    source: self.transit_source_account.to_account_info(),
                    destination: self.source_account.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                },
            ),
            ChangeKind::ShiftBy,
            amount,
        )?;

        Ok(())
    }

    // todo fixme
    // amount: new u64(10),
    // otherAmountThreshold: ZERO_BN,
    // sqrtPriceLimit: new anchor.BN(MAX_SQRT_PRICE),
    // amountSpecifiedIsInput: false,
    // aToB: false,

    // whirlpool: whirlpoolPda.publicKey,
    // tokenAuthority: ctx.wallet.publicKey,
    // tokenOwnerAccountA: tokenAccountA,
    // tokenVaultA: poolInitInfo.tokenVaultAKeypair.publicKey,
    // tokenOwnerAccountB: tokenAccountB,
    // tokenVaultB: poolInitInfo.tokenVaultBKeypair.publicKey,
    // tickArray0: tickArrays[0].publicKey,
    // tickArray1: tickArrays[0].publicKey,
    // tickArray2: tickArrays[0].publicKey,
    // oracle: oraclePda.publicKey,

    /// Issue a Orca swap
    #[inline(never)]
    fn swap(
        &self,
        withdrawal_amount: u64,
        other_amount_threshold: u64,
        sqrt_price_limit: u128,
        amount_specified_is_input: bool,
        a_to_b: bool,
    ) -> Result<()> {
        let swap_context = CpiContext::new(
            self.token_program.to_account_info(),
            // self.swap_program.to_account_info(),
            orca_whirlpool::Swap {
                token_program: self.token_program.to_account_info(),
                token_authority: self.margin_account.to_account_info(),
                whirlpool: self.swap_info.whirlpool.to_account_info(),
                token_owner_account_a: self.transit_source_account.to_account_info(),
                token_vault_a: self.swap_info.vault_a.to_account_info(),
                token_owner_account_b: self.transit_destination_account.to_account_info(),
                token_vault_b: self.swap_info.vault_b.to_account_info(),
                tick_array_0: self.swap_info.tick_array_0.to_account_info(),
                tick_array_1: self.swap_info.tick_array_1.to_account_info(),
                tick_array_2: self.swap_info.tick_array_2.to_account_info(),
                oracle: self.swap_info.oracle.to_account_info(),
            },
        );

        orca_whirlpool::swap(
            swap_context,
            withdrawal_amount,
            other_amount_threshold,
            sqrt_price_limit,
            amount_specified_is_input,
            a_to_b,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct WhirlpoolSwapInfo<'info> {
    /// Checked by Orca, whirlpool state
    pub whirlpool: UncheckedAccount<'info>,

    /// Checked by Orca
    pub token_authority: UncheckedAccount<'info>,

    /// Checked by Orca
    #[account(mut)]
    pub vault_a: UncheckedAccount<'info>,

    /// Checked by Orca
    #[account(mut)]
    pub vault_b: UncheckedAccount<'info>,

    /// Checked by Orca
    #[account(mut)]
    pub tick_array_0: UncheckedAccount<'info>,
    /// Checked by Orca
    #[account(mut)]
    pub tick_array_1: UncheckedAccount<'info>,
    /// Checked by Orca
    #[account(mut)]
    pub tick_array_2: UncheckedAccount<'info>,
    /// Checked by Orca
    /// Oracle is currently unused and will be enabled on subsequent updates
    pub oracle: UncheckedAccount<'info>,
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
pub fn orca_whirlpool_swap_handler(
    ctx: Context<OrcaWhirlpoolSwapPool>,
    withdrawal_change_kind: ChangeKind,
    withdrawal_amount: u64,
    other_amount_threshold: u64,
    sqrt_price_limit: u128,
    amount_specified_is_input: bool,
    a_to_b: bool, // Zero for one
) -> Result<()> {
    // TODO: this structure can be repeated across swap instructions, DRY
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
    ctx.accounts.swap(
        withdrawal_amount,
        other_amount_threshold,
        sqrt_price_limit,
        amount_specified_is_input,
        a_to_b,
    )?;

    let destination_closing_balance =
        token::accessor::amount(&ctx.accounts.transit_destination_account.to_account_info())?;

    // If the swap would have resulted in 0 tokens, the swap program would error out,
    // thus balance below will be positive.
    let swap_amount_out = destination_closing_balance
        .checked_sub(destination_opening_balance)
        .unwrap();
    ctx.accounts.deposit_destination(swap_amount_out)?;

    // check if there was less required for the min amount out then expected
    // aka if there was unexpected leftover balance in the source account after
    // the swap occurred.

    let source_amount_after_swap =
        token::accessor::amount(&ctx.accounts.transit_source_account.to_account_info())?;

    let leftover_balance_from_source_account = source_amount_after_swap
        .checked_sub(source_opening_balance)
        .unwrap();

    // if there was leftover balance in the source transit account, deposit into the pool
    if leftover_balance_from_source_account > 0 {
        ctx.accounts
            .deposit_source(leftover_balance_from_source_account)?;
    }

    Ok(())
}
