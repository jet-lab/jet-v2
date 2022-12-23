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

use std::{
    collections::{BTreeMap, BTreeSet},
    slice::Iter,
};

use anchor_spl::token::Token;
use jet_margin_pool::ChangeKind;

use crate::*;

#[derive(Accounts)]
pub struct RouteSwap<'info> {
    /// The margin account being executed on
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The account with the source deposit to be exchanged from
    /// CHECK: The pool program validates the authority of the withdrawal
    #[account(mut)]
    pub source_account: AccountInfo<'info>,

    /// The destination account to send the deposit that is exchanged into
    /// CHECK: The token program validates both type and ownership thorugh withdrawals.
    /// The swap is also atomic, and no excess funds would be taken/left in the account.
    #[account(mut)]
    pub destination_account: UncheckedAccount<'info>,

    /// Temporary account for moving tokens
    /// CHECK: The token program validates both type and ownership thorugh withdrawals.
    /// The swap is also atomic, and no excess funds would be taken/left in the account.
    #[account(mut)]
    pub transit_source_account: UncheckedAccount<'info>,

    /// Temporary account for moving tokens
    /// CHECK: The token program validates both type and ownership thorugh withdrawals.
    /// The swap is also atomic, and no excess funds would be taken/left in the account.
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
}

/// Route a swap with up to 3 legs, which can be split across venues (e.g 80/20).
///
/// The instruction relies on extra accounts, which are structured for each leg as:
/// - associated token account
/// - 3 pool accounts (margin_pool, vault, deposit_note_mint)
/// - deposit note account
/// - accounts of the swap instruction
///
/// Where there are multiple swaps, the above are concatenated to each other
pub fn route_swap_handler<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, RouteSwap<'info>>,
    withdrawal_change_kind: ChangeKind,
    withdrawal_amount: u64,
    minimum_amount_out: u64,
    swap_routes: [SwapRouteDetail; 3],
) -> Result<()> {
    // To protect users, the minimum_amount_out should always be positive.
    // We only check for slippage after all swaps, and some swaps might return 0
    // tokens, so we prevent this by ensuring that we'll compare against > 0.
    assert!(minimum_amount_out > 0);
    // Validate input and find out how many swaps there are
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
    let mut swap_amount_in = source_closing_balance
        .checked_sub(source_opening_balance)
        .unwrap();
    if swap_amount_in == 0 {
        return err!(crate::ErrorCode::NoSwapTokensWithdrawn);
    }

    // let mut account_index = 0;
    let mut remaining_accounts = ctx.remaining_accounts.iter();

    // Iterate through all the valid swap legs and execute the swaps
    for (leg, route) in swap_routes.iter().enumerate().take(valid_swaps) {
        let (source_pool_accounts, src_transit, source_pool_dep_note) = match leg {
            0 => {
                // First leg of 1 or more
                // The source accounts are from the source_margin_pool

                // 3 accounts are used
                // - margin_pool
                // - vault
                // - deposit_note_mint
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

                let source_ata = next_account_info(&mut remaining_accounts)?.to_account_info();

                // More convenient than next_account_infos(_, 3) as we want an array
                let source_pool_accounts = [
                    next_account_info(&mut remaining_accounts)?.to_account_info(),
                    next_account_info(&mut remaining_accounts)?.to_account_info(),
                    next_account_info(&mut remaining_accounts)?.to_account_info(),
                ];

                let source_pool_dep_note =
                    next_account_info(&mut remaining_accounts)?.to_account_info();

                (source_pool_accounts, source_ata, source_pool_dep_note)
            }
        };
        // If this is the last/only leg, the destination transit is known, else get after swap accounts
        let dst_transit = if leg + 1 == valid_swaps {
            Some(&ctx.accounts.transit_destination_account)
        } else {
            None
        };
        let amount_in = exec_swap(
            &ctx,
            route,
            &source_pool_accounts,
            &src_transit,
            dst_transit,
            &source_pool_dep_note,
            &mut remaining_accounts,
            swap_amount_in,
        )?;
        swap_amount_in = amount_in;
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
        msg!(
            "Amount out = {} less than minimum {}",
            swap_amount_out,
            minimum_amount_out
        );
        return Err(error!(crate::ErrorCode::SlippageExceeded));
    }

    // Deposit into the destination pool
    ctx.accounts.deposit(
        dest_pool_accounts,
        &destination_ata,
        &destination_pool_dep_note,
        ChangeKind::ShiftBy,
        swap_amount_out,
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn exec_swap<'a, 'b, 'c, 'info>(
    ctx: &Context<'a, 'b, 'c, 'info, RouteSwap<'info>>,
    route: &SwapRouteDetail,
    source_pool_accounts: &[AccountInfo<'info>; 3],
    src_ata: &AccountInfo<'info>,
    dst_ata_opt: Option<&AccountInfo<'info>>,
    source_pool_dep_note: &AccountInfo<'info>,
    remaining_accounts: &mut Iter<AccountInfo<'info>>,
    swap_amount_in: u64,
) -> Result<u64> {
    // CHECK: The token program will withdraw from this account, and will check
    // that the type and authority are correct.
    let src_ata_opening = token::accessor::amount(src_ata)?;
    // Record the opening balance of the input
    //
    // Get the swap accounts

    // Get the amount for the current leg if there is a split
    let curr_swap_in = if route.split == 0 {
        swap_amount_in
    } else {
        // This is safe as we have checked that split < 100 when validating legs
        (swap_amount_in * route.split as u64) / 100
    };

    // Get the ATA opening and closing balances
    let (dst_ata_opening, mut dst_ata_closing) = exec_swap_split(
        &ctx.accounts.margin_account.to_account_info(),
        &ctx.accounts.token_program,
        &route.route_a,
        src_ata,
        dst_ata_opt,
        remaining_accounts,
        curr_swap_in,
    )?;

    // Handle the next leg
    if route.split > 0 {
        // Get the remaining amount to swap
        let curr_swap_in = swap_amount_in.checked_sub(curr_swap_in).unwrap();
        assert!(curr_swap_in > 0);

        let (_, closing) = exec_swap_split(
            &ctx.accounts.margin_account.to_account_info(),
            &ctx.accounts.token_program,
            &route.route_a,
            src_ata,
            dst_ata_opt,
            remaining_accounts,
            curr_swap_in,
        )?;
        // overwrite the dst_ata_closing with its latest balance
        dst_ata_closing = closing;
    }

    // After the swaps above, we can now return any dust to the input token's pool
    let src_ata_closing = token::accessor::amount(src_ata)?;
    // Track how much was swapped, the balance between expected vs actual is returned
    // to the relevant pool
    let total_swap_input = src_ata_opening.checked_sub(src_ata_closing).unwrap();
    // Track how much should be withdrawn from the ATA account in the next swap.
    let total_swap_output = dst_ata_closing.checked_sub(dst_ata_opening).unwrap();

    if total_swap_input < swap_amount_in {
        // We swapped less than we had to, return dust back
        let remaining_bal = swap_amount_in - total_swap_input;
        ctx.accounts.deposit(
            source_pool_accounts,
            src_ata,
            source_pool_dep_note,
            ChangeKind::ShiftBy,
            remaining_bal,
        )?;
    }

    Ok(total_swap_output)
}

/// Execute the route leg and return the opening and closing balance of the ATA used
#[inline]
fn exec_swap_split<'info>(
    authority: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    route_ident: &SwapRouteIdentifier,
    src_ata: &AccountInfo<'info>,
    dst_ata_opt: Option<&AccountInfo<'info>>,
    remaining_accounts: &mut Iter<AccountInfo<'info>>,
    swap_amount_in: u64,
) -> Result<(u64, u64)> {
    let dst_ata_opening: u64;
    let dst_ata_closing: u64;
    let mut bumps = BTreeMap::new();
    let mut reallocs = BTreeSet::new();
    let mut accounts = remaining_accounts.as_slice();
    match route_ident {
        SwapRouteIdentifier::Empty => return Err(error!(crate::ErrorCode::InvalidSwapRoute)),
        SwapRouteIdentifier::Spl => {
            let swap_accounts = SplSwapInfo::try_accounts(
                // &program_id,
                // TODO: how can we get the program_id?
                &Pubkey::default(),
                &mut accounts,
                &[],
                &mut bumps,
                &mut reallocs,
            )?;
            // We don't need to check the destination balance on this leg
            let dst_ata =
                dst_ata_opt.unwrap_or_else(|| next_account_info(remaining_accounts).unwrap());
            dst_ata_opening = token::accessor::amount(dst_ata)?;

            swap_accounts.swap(
                src_ata,
                dst_ata,
                &authority.to_account_info(),
                token_program,
                swap_amount_in,
                0,
            )?;
            dst_ata_closing = token::accessor::amount(dst_ata)?;
        }
        SwapRouteIdentifier::Whirlpool => todo!(),
        SwapRouteIdentifier::SaberStable => {
            let swap_accounts = SaberSwapInfo::try_accounts(
                &saber_stable_swap::id(),
                &mut accounts,
                &[],
                &mut bumps,
                &mut reallocs,
            )?;
            // We don't need to check the destination balance on this leg
            let dst_ata =
                dst_ata_opt.unwrap_or_else(|| next_account_info(remaining_accounts).unwrap());
            dst_ata_opening = token::accessor::amount(dst_ata)?;

            swap_accounts.swap(
                src_ata,
                dst_ata,
                &authority.to_account_info(),
                token_program,
                swap_amount_in,
                0,
            )?;
            dst_ata_closing = token::accessor::amount(dst_ata)?;
        }
    };

    Ok((dst_ata_opening, dst_ata_closing))
}
