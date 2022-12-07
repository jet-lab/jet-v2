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
pub struct RouteSwapPool<'info> {
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
    pub destination_account: AccountInfo<'info>,

    /// The accounts relevant to the source margin pool
    pub source_margin_pool: MarginPoolInfo<'info>,

    /// The accounts relevant to the destination margin pool
    pub destination_margin_pool: MarginPoolInfo<'info>,

    pub margin_pool_program: Program<'info, JetMarginPool>,

    pub token_program: Program<'info, Token>,
}

impl<'info> RouteSwapPool<'info> {
    #[inline(never)]
    fn withdraw(
        &self,
        change_kind: ChangeKind,
        amount_in: u64,
        destination: &AccountInfo<'info>,
    ) -> Result<()> {
        jet_margin_pool::cpi::withdraw(
            CpiContext::new(
                self.margin_pool_program.to_account_info(),
                Withdraw {
                    margin_pool: self.source_margin_pool.margin_pool.to_account_info(),
                    vault: self.source_margin_pool.vault.to_account_info(),
                    deposit_note_mint: self.source_margin_pool.deposit_note_mint.to_account_info(),
                    depositor: self.margin_account.to_account_info(),
                    source: self.source_account.to_account_info(),
                    destination: destination.to_account_info(),
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

#[derive(Accounts)]
pub struct RouteSwap<'info> {
    /// The margin account being executed on
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    pub token_program: Program<'info, Token>,
}

/// Route a swap with up to 3 legs, which can be split across venues (e.g 80/20).
///
/// The instruction relies on extra accounts, which are structured for each leg as:
/// - associated token account
/// - accounts of the swap instruction
///
/// Where there are multiple swaps, the above are concatenated to each other
pub fn route_swap_pool_handler<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, RouteSwapPool<'info>>,
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

    let mut remaining_accounts = ctx.remaining_accounts.iter();
    let (mut src_transit, dst_transit) = {
        let slice = remaining_accounts.as_slice();
        (
            remaining_accounts.next().unwrap().to_account_info(),
            slice.last().unwrap(),
        )
    };
    let source_opening_balance = token::accessor::amount(&src_transit)?;
    // Withdraw from pool into ATA
    ctx.accounts
        .withdraw(withdrawal_change_kind, withdrawal_amount, &src_transit)?;
    let source_closing_balance = token::accessor::amount(&src_transit)?;

    let destination_opening_balance = token::accessor::amount(dst_transit)?;

    // The closing balance should be > opening balance after the withdrawal
    let mut swap_amount_in = source_closing_balance
        .checked_sub(source_opening_balance)
        .unwrap();
    if swap_amount_in == 0 {
        return err!(crate::ErrorCode::NoSwapTokensWithdrawn);
    }

    // Iterate through all the valid swap legs and execute the swaps
    for route in swap_routes.iter().take(valid_swaps) {
        let (amount_in, next_src_transit) = exec_swap(
            &ctx.accounts.margin_account.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            &src_transit,
            &mut remaining_accounts,
            route,
            swap_amount_in,
        )?;
        swap_amount_in = amount_in;
        src_transit = next_src_transit;
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

    let destination_pool_dep_note = ctx.accounts.destination_account.to_account_info();

    let destination_closing_balance = token::accessor::amount(dst_transit)?;

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
        dst_transit,
        &destination_pool_dep_note,
        ChangeKind::ShiftBy,
        swap_amount_out,
    )?;

    Ok(())
}

pub fn route_swap_handler<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, RouteSwap<'info>>,
    amount_in: u64,
    minimum_amount_out: u64,
    swap_routes: [SwapRouteDetail; 3],
) -> Result<()> {
    // To protect users, the minimum_amount_out should always be positive.
    // We only check for slippage after all swaps, and some swaps might return 0
    // tokens, so we prevent this by ensuring that we'll compare against > 0.
    assert!(amount_in > 0 && minimum_amount_out > 0);
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

    let mut remaining_accounts = ctx.remaining_accounts.iter();
    let (mut src_transit, dst_transit) = {
        let slice = remaining_accounts.as_slice();
        (
            remaining_accounts.next().unwrap().to_account_info(),
            slice.last().unwrap(),
        )
    };

    // The destination opening balance is used to track how many tokens were swapped
    let destination_opening_balance = token::accessor::amount(dst_transit)?;

    let mut swap_amount_in = amount_in;

    // Iterate through all the valid swap legs and execute the swaps
    for route in swap_routes.iter().take(valid_swaps) {
        let (amount_in, next_src_transit) = exec_swap(
            &ctx.accounts.margin_account.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            &src_transit,
            &mut remaining_accounts,
            route,
            swap_amount_in,
        )?;
        swap_amount_in = amount_in;
        src_transit = next_src_transit;
    }

    let destination_closing_balance = token::accessor::amount(dst_transit)?;

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

    Ok(())
}

/// Execute a swap and return number of tokens swapped and the destination account
fn exec_swap<'info>(
    margin_account: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    src_ata: &AccountInfo<'info>,
    remaining_accounts: &mut Iter<AccountInfo<'info>>,
    route: &SwapRouteDetail,
    swap_amount_in: u64,
) -> Result<(u64, AccountInfo<'info>)> {
    // Get the amount for the current leg if there is a split
    let curr_swap_in = if route.split == 0 {
        swap_amount_in
    } else {
        // This is safe as we have checked that split < 100 when validating legs
        (swap_amount_in * route.split as u64) / 100
    };

    // Get the ATA opening and closing balances
    let (dst_ata_opening, mut dst_ata_closing, dst_transit) = exec_swap_split(
        margin_account,
        token_program,
        &route.route_a,
        src_ata,
        remaining_accounts,
        curr_swap_in,
    )?;

    // Handle the next leg
    if route.split > 0 {
        // Get the remaining amount to swap
        let curr_swap_in = swap_amount_in.checked_sub(curr_swap_in).unwrap();
        assert!(curr_swap_in > 0);

        let (_, closing, dst) = exec_swap_split(
            margin_account,
            token_program,
            &route.route_a,
            src_ata,
            remaining_accounts,
            curr_swap_in,
        )?;
        // overwrite the dst_ata_closing with its latest balance
        dst_ata_closing = closing;
        if dst_transit.key != dst.key {
            todo!("Tokens in a swap split should go to the same destination account");
        }
    }

    Ok((
        dst_ata_closing.checked_sub(dst_ata_opening).unwrap(),
        dst_transit,
    ))
}

/// Execute the route leg and return the opening and closing balance of the ATA used
#[inline]
fn exec_swap_split<'info>(
    authority: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    route_ident: &SwapRouteIdentifier,
    src_ata: &AccountInfo<'info>,
    remaining_accounts: &mut Iter<AccountInfo<'info>>,
    swap_amount_in: u64,
) -> Result<(u64, u64, AccountInfo<'info>)> {
    let dst_ata_opening: u64;
    let dst_ata_closing: u64;
    let mut bumps = BTreeMap::new();
    let mut reallocs = BTreeSet::new();
    let dst_ata = match route_ident {
        SwapRouteIdentifier::Empty => return Err(error!(crate::ErrorCode::InvalidSwapRoute)),
        SwapRouteIdentifier::Spl => {
            let accounts = remaining_accounts.take(7).cloned().collect::<Vec<_>>();
            let swap_accounts = SplSwapInfo::try_accounts(
                // Will be validated by the spl swap registry
                &Pubkey::default(),
                &mut &accounts[..],
                &[],
                &mut bumps,
                &mut reallocs,
            )?;
            // We don't need to check the destination balance on this leg
            let dst_ata = next_account_info(remaining_accounts).unwrap();
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

            dst_ata.to_account_info()
        }
        SwapRouteIdentifier::Whirlpool => todo!(),
        SwapRouteIdentifier::SaberStable => {
            let accounts = remaining_accounts.take(6).cloned().collect::<Vec<_>>();
            let swap_accounts = SaberSwapInfo::try_accounts(
                &saber_stable_swap::id(),
                &mut &accounts[..],
                &[],
                &mut bumps,
                &mut reallocs,
            )?;
            // We don't need to check the destination balance on this leg
            let dst_ata = next_account_info(remaining_accounts).unwrap();
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

            dst_ata.to_account_info()
        }
    };

    Ok((dst_ata_opening, dst_ata_closing, dst_ata))
}
