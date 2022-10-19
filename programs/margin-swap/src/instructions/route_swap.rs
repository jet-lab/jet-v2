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

use anchor_lang::solana_program::account_info::next_account_infos;
use anchor_spl::token::Token;
use jet_margin_pool::ChangeKind;
use jet_static_program_registry::{orca_swap_v1, orca_swap_v2, spl_token_swap_v2};

use crate::*;

#[derive(Accounts)]
pub struct RouteSwap<'info> {
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

    /// The accounts relevant to the source margin pool
    pub source_margin_pool: MarginPoolInfo<'info>,

    /// The accounts relevant to the destination margin pool
    pub destination_margin_pool: MarginPoolInfo<'info>,

    pub margin_pool_program: Program<'info, JetMarginPool>,

    pub token_program: Program<'info, Token>,
}

impl<'info> RouteSwap<'info> {
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

    /// Deposit to the destination pool of the swap
    #[inline(never)]
    fn deposit(&self, amount: u64) -> Result<()> {
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
    fn spl_swap(
        &self,
        accounts: &[AccountInfo<'info>],
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        assert!(accounts.len() >= 7);
        let swap_pool = &accounts[0];
        let authority = &accounts[1];
        let vault_into = &accounts[2];
        let vault_from = &accounts[3];
        let token_mint = &accounts[4];
        let fee_account = &accounts[5];
        // CHECK: The swap program gets validated by use_client! below
        let swap_program = &accounts[6];

        let swap_ix = use_client!(swap_program.key(), {
            client::instruction::swap(
                swap_program.key,
                self.token_program.key,
                swap_pool.key,
                authority.key,
                &self.margin_account.key(),
                self.transit_source_account.key,
                vault_into.key,
                vault_from.key,
                self.transit_destination_account.key,
                token_mint.key,
                fee_account.key,
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
                swap_pool.to_account_info(),
                self.margin_account.to_account_info(),
                authority.to_account_info(),
                self.transit_source_account.to_account_info(),
                vault_into.to_account_info(),
                vault_from.to_account_info(),
                self.transit_destination_account.to_account_info(),
                token_mint.to_account_info(),
                fee_account.to_account_info(),
                self.token_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    #[inline(never)]
    fn saber_stable_swap(
        &self,
        accounts: &[AccountInfo<'info>],
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        assert!(accounts.len() >= 7);
        let swap_pool = &accounts[0];
        let authority = &accounts[1];
        let vault_into = &accounts[2];
        let vault_from = &accounts[3];
        let token_mint = &accounts[4];
        let fee_account = &accounts[5];
        let swap_program = &accounts[6];

        // TODO program error
        assert_eq!(swap_program.key(), saber_stable_swap::id());

        let swap_context = CpiContext::new(
            self.token_program.to_account_info(),
            saber_stable_swap::Swap {
                user: saber_stable_swap::SwapUserContext {
                    token_program: self.token_program.to_account_info(),
                    swap_authority: authority.to_account_info(),
                    user_authority: self.margin_account.to_account_info(),
                    swap: swap_pool.to_account_info(),
                },
                input: saber_stable_swap::SwapToken {
                    user: self.transit_source_account.to_account_info(),
                    reserve: vault_into.to_account_info(),
                },
                output: saber_stable_swap::SwapOutput {
                    user_token: saber_stable_swap::SwapToken {
                        user: self.transit_destination_account.to_account_info(),
                        reserve: vault_from.to_account_info(),
                    },
                    fees: fee_account.to_account_info(),
                },
            },
        );

        saber_stable_swap::swap(swap_context, amount_in, minimum_amount_out)?;

        Ok(())
    }
}

pub fn route_swap_handler<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, RouteSwap<'info>>,
    withdrawal_change_kind: ChangeKind,
    withdrawal_amount: u64,
    minimum_amount_out: u64,
    swap_routes: [SwapRouteDetail; 3],
) -> Result<()> {
    // Validate input and find out how many swaps there are
    // TODO: measure compute impact
    let valid_swaps = match (
        swap_routes[0].validate()?,
        swap_routes[1].validate()?,
        swap_routes[2].validate()?,
    ) {
        (_, false, true) | (false, _, _) => return Err(error!(crate::ErrorCode::InvalidSwapRoute)),
        (true, true, true) => 3,
        (true, true, false) => 2,
        (true, false, _) => 1,
    };

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

    // Execute the first swap
    let route = &swap_routes[0];
    // Don't process split paths in current iteration
    if route.split > 0 {
        return err!(crate::ErrorCode::InvalidSwapRouteParam);
    }

    let swap_min_out = if valid_swaps == 1 {
        minimum_amount_out
    } else {
        0
    };

    let mut remaining_accounts = ctx.remaining_accounts.iter();

    match route.route_a {
        SwapRouteIdentifier::Empty => unreachable!(),
        SwapRouteIdentifier::Spl => {
            let swap_accounts = next_account_infos(&mut remaining_accounts, 7)?;
            ctx.accounts
                .spl_swap(swap_accounts, swap_amount_in, swap_min_out)?;
        }
        SwapRouteIdentifier::Whirlpool => todo!(),
        SwapRouteIdentifier::SaberStable => {
            let swap_accounts = next_account_infos(&mut remaining_accounts, 7)?;
            ctx.accounts
                .saber_stable_swap(swap_accounts, swap_amount_in, swap_min_out)?;
        }
    }

    let destination_closing_balance =
        token::accessor::amount(&ctx.accounts.transit_destination_account.to_account_info())?;
    // If the swap would have resulted in 0 tokens, the swap program would error out,
    // thus balance below will be positive.
    let swap_amount_out = destination_closing_balance
        .checked_sub(destination_opening_balance)
        .unwrap();
    // Deposit back into the pool
    ctx.accounts.deposit(swap_amount_out)?; // TODO: specify accounts

    // Return any interim and source dust
    let source_amount_after_swap =
        token::accessor::amount(&ctx.accounts.transit_source_account.to_account_info())?;

    let leftover_balance_from_source_account = source_amount_after_swap
        .checked_sub(source_opening_balance)
        .unwrap();

    // if there was leftover balance in the source transit account, deposit into the pool
    if leftover_balance_from_source_account > 0 {
        ctx.accounts.deposit(leftover_balance_from_source_account)?; // TODO: specify accounts
    }

    Ok(())
}
