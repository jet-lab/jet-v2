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
    fn deposit(
        &self,
        pool_accounts: &[AccountInfo<'info>; 3],
        source: &AccountInfo<'info>,
        destination: &AccountInfo<'info>,
        change_kind: ChangeKind,
        amount: u64,
    ) -> Result<()> {
        let margin_pool = &pool_accounts[0];
        let vault = &pool_accounts[1];
        let note_mint = &pool_accounts[2];
        jet_margin_pool::cpi::deposit(
            CpiContext::new(
                self.margin_pool_program.to_account_info(),
                Deposit {
                    margin_pool: margin_pool.to_account_info(),
                    vault: vault.to_account_info(),
                    deposit_note_mint: note_mint.to_account_info(),
                    depositor: self.margin_account.to_account_info(),
                    source: source.to_account_info(),
                    destination: destination.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                },
            ),
            change_kind,
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

    // In all cases we first withdraw from the source pool

    // Get the balance before the withdrawal. The balance should almost always
    // be zero, however it could already have a value.
    let source_opening_balance =
        token::accessor::amount(&ctx.accounts.transit_source_account.to_account_info())?;
    let destination_opening_balance =
        token::accessor::amount(&ctx.accounts.transit_destination_account.to_account_info())?;

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

    let mut acc_ix = 0;
    // TODO: will use for bounds checking
    let _remaining_accounts_len = ctx.remaining_accounts.len();

    // TODO: went the long way so I can find a pattern as I go along
    for leg in 0..valid_swaps {
        let route = &swap_routes[leg];
        let (source_pool_accounts, source_ata, source_pool_dep_note) = match leg {
            0 => {
                // First leg of 2 or more

                let source_pool_accounts = [
                    ctx.accounts
                        .source_margin_pool
                        .margin_pool
                        .to_account_info(),
                    ctx.accounts.source_margin_pool.vault.to_account_info(),
                    ctx.accounts
                        .source_margin_pool
                        .deposit_note_mint
                        .to_account_info(),
                ];

                let source_ata = ctx.accounts.transit_source_account.to_account_info();
                let source_pool_dep_note = ctx.accounts.source_account.to_account_info();

                (source_pool_accounts, source_ata, source_pool_dep_note)
            }
            _ => {
                // Not the first leg

                let source_pool_accounts = [
                    ctx.remaining_accounts[acc_ix].to_account_info(),
                    ctx.remaining_accounts[acc_ix + 1].to_account_info(),
                    ctx.remaining_accounts[acc_ix + 2].to_account_info(),
                ];

                let source_ata = ctx.remaining_accounts[acc_ix + 3].to_account_info();
                let source_pool_dep_note = ctx.remaining_accounts[acc_ix + 4].to_account_info();
                acc_ix += 5;

                (source_pool_accounts, source_ata, source_pool_dep_note)
            }
        };
        acc_ix = exec_swap(
            &ctx,
            route,
            &source_pool_accounts,
            &source_ata,
            &source_pool_dep_note,
            swap_amount_in,
            acc_ix,
        )?;
    }

    // If the swap would have resulted in 0 tokens, the swap program would error out,
    // thus balance below will be positive.

    let dest_pool_accounts = &[
        ctx.accounts
            .destination_margin_pool
            .margin_pool
            .to_account_info(),
        ctx.accounts.destination_margin_pool.vault.to_account_info(),
        ctx.accounts
            .destination_margin_pool
            .deposit_note_mint
            .to_account_info(),
    ];

    let destination_ata = ctx.accounts.transit_destination_account.to_account_info();
    let destination_pool_dep_note = ctx.accounts.destination_account.to_account_info();

    let destination_closing_balance =
        token::accessor::amount(&ctx.accounts.transit_destination_account.to_account_info())?;

    let swap_amount_out = destination_closing_balance
        .checked_sub(destination_opening_balance)
        .unwrap();
    // Check if slippage tolerance is exceeded
    if swap_amount_out < minimum_amount_out {
        msg!("Amount out = {swap_amount_out} less than minimum {minimum_amount_out}");
        return Err(error!(crate::ErrorCode::SlippageExceeded));
    }
    ctx.accounts.deposit(
        dest_pool_accounts,
        &destination_ata,
        &destination_pool_dep_note,
        ChangeKind::ShiftBy,
        swap_amount_out,
    )?;

    Ok(())
}

fn exec_swap<'a, 'b, 'c, 'info>(
    ctx: &Context<'a, 'b, 'c, 'info, RouteSwap<'info>>,
    route: &SwapRouteDetail,
    source_pool_accounts: &[AccountInfo<'info>; 3],
    source_ata: &AccountInfo<'info>,
    source_pool_dep_note: &AccountInfo<'info>,
    swap_amount_in: u64,
    acc_ix: usize,
) -> Result<usize> {
    let mut acc_ix = acc_ix;
    let source_ata_opening = token::accessor::amount(source_ata)?;
    // Record the opening balance of the input
    //
    // Get the swap accounts
    // TODO: is it better to use next_accounts? That requires advancing the iterator

    // Get the amount for the current leg if there is a split
    let curr_swap_in = if route.split == 0 {
        swap_amount_in
    } else {
        // This is safe as we have checked that split < 100
        (swap_amount_in * route.split as u64) / 100
    };

    // TODO: handle splits
    match route.route_a {
        SwapRouteIdentifier::Empty => todo!(),
        SwapRouteIdentifier::Spl => {
            // TODO: add bounds check
            let accounts = &ctx.remaining_accounts[acc_ix..acc_ix + 7];
            acc_ix += 7;
            // We don't need to check the destination balance on this leg
            // TODO: handle percentages
            ctx.accounts.spl_swap(accounts, curr_swap_in, 0)?;
        }
        SwapRouteIdentifier::Whirlpool => todo!(),
        SwapRouteIdentifier::SaberStable => todo!(),
    }

    // Handle the next leg
    if route.split > 0 {
        let curr_swap_in = swap_amount_in.checked_sub(curr_swap_in).unwrap();
        assert!(curr_swap_in > 0); // TODO: limit split to 90 to avoid too small amounts

        match route.route_b {
            SwapRouteIdentifier::Empty => todo!(),
            SwapRouteIdentifier::Spl => {
                // TODO: add bounds check
                let accounts = &ctx.remaining_accounts[acc_ix..acc_ix + 7];
                acc_ix += 7;
                // We don't need to check the destination balance on this leg
                // TODO: handle percentages
                ctx.accounts.spl_swap(accounts, curr_swap_in, 0)?;
            }
            SwapRouteIdentifier::Whirlpool => todo!(),
            SwapRouteIdentifier::SaberStable => todo!(),
        }
    }

    // After the swaps above, we can now return any dust to the input token's pool
    // TODO: if we've taken a reference, does the token amount change?
    let source_ata_closing = token::accessor::amount(source_ata)?;
    let swapped_amount = source_ata_opening.checked_sub(source_ata_closing).unwrap();

    if swapped_amount < swap_amount_in {
        // We swapped less than we had to, return dust back
        let remaining_bal = swap_amount_in - swapped_amount;
        ctx.accounts.deposit(
            source_pool_accounts,
            source_ata,
            source_pool_dep_note,
            ChangeKind::ShiftBy,
            remaining_bal,
        )?;
    }

    Ok(acc_ix)
}
