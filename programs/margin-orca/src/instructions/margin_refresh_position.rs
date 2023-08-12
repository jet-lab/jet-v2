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

use anchor_lang::prelude::*;

use jet_margin::MarginAccount;
use orca_whirlpool::state::Position;

use crate::*;

pub const POSITION_VALUE_EXPO: i32 = -8;

#[derive(Accounts)]
pub struct MarginRefreshPosition<'info> {
    /// The margin account being executed on
    pub owner: AccountLoader<'info, MarginAccount>,

    /// The pool to be refreshed
    #[account(
        has_one = token_a_oracle,
        has_one = token_b_oracle
    )]
    pub whirlpool_config: Box<Account<'info, WhirlpoolConfig>>,

    #[account(mut, has_one = owner)]
    pub adapter_position_metadata: Box<Account<'info, PositionMetadata>>,

    /// The pyth price account for the position's token A
    /// CHECK:
    pub token_a_oracle: AccountInfo<'info>,

    /// The pyth price account for the position's token B
    /// CHECK:
    pub token_b_oracle: AccountInfo<'info>,
    // Whirlpools and positions are passed in as remaining_accounts
}

/// Refresh the position by calculating the entitled tokens and valuing them
/// based on their oracle prices.
/// This instruction should be called after the underlying positions have been
/// updated with `update_fees_and_rewards`.
///
/// If the positions have not been refreshed prior, the main implication is that
/// fees and rewards will not be recent. This is not an issue as they are expected
/// to be smaller than the position itself. There is no risk of the position being
/// stale as we use the latest whirlpool balance to use its price.
pub fn margin_refresh_position_handler(ctx: Context<MarginRefreshPosition>) -> Result<()> {
    // Cache oracle prices
    ctx.accounts
        .adapter_position_metadata
        .update_oracle_prices(&ctx.accounts.token_a_oracle, &ctx.accounts.token_b_oracle)?;

    let clock = Clock::get()?;
    let timestamp = clock.unix_timestamp;

    let total_whirlpools = ctx.accounts.adapter_position_metadata.total_whirlpools();

    let mut remaining_accounts = ctx.remaining_accounts.iter();
    // Update whirlpool prices
    for account in remaining_accounts.by_ref().take(total_whirlpools) {
        let whirlpool = Account::<orca_whirlpool::state::Whirlpool>::try_from(account)?;
        ctx.accounts
            .adapter_position_metadata
            .update_whirlpool_prices(&whirlpool, timestamp);
    }

    // Update whirlpool position balances and accrued fees
    for account in remaining_accounts {
        let whirlpool_position = Account::<Position>::try_from(account)?;
        ctx.accounts
            .adapter_position_metadata
            .update_position(&whirlpool_position)?;
    }

    // Tell the margin program what the current prices are
    ctx.accounts
        .adapter_position_metadata
        .update_position_balance(&*ctx.accounts.owner.load()?, &ctx.accounts.whirlpool_config)
}
