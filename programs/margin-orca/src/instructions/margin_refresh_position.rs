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
    collections::{HashMap, HashSet},
    convert::TryFrom,
    iter::FromIterator,
};

use anchor_lang::prelude::*;

use jet_margin::{AdapterResult, MarginAccount, PositionChange, PriceChangeInfo};
use jet_program_common::{
    traits::{SafeDiv, SafeMul},
    Number128,
};
use orca_whirlpool::state::Position;

use crate::*;

pub const POSITION_VALUE_EXPO: i32 = -8;
/// The percentage tolerance for the price of a whirlpool to be considered valid.
/// The percentage is in units, such that 1 = 1%.
pub const PRICE_TICK_TOLERANCE: i32 = 3;

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
    let info = &ctx.accounts.whirlpool_config;
    let meta = &ctx.accounts.adapter_position_metadata;

    let total_whirlpools = meta
        .positions()
        .into_iter()
        .map(|p| p.whirlpool)
        .collect::<HashSet<_>>()
        .len();

    let token_a_oracle =
        match pyth_sdk_solana::load_price_feed_from_account_info(&ctx.accounts.token_a_oracle) {
            Ok(pf) => pf,
            Err(e) => {
                msg!("the oracle account is not valid: {:?}", e);
                return err!(MarginOrcaErrorCode::InvalidOracle);
            }
        };
    // TODO: DRY
    let token_b_oracle =
        match pyth_sdk_solana::load_price_feed_from_account_info(&ctx.accounts.token_b_oracle) {
            Ok(pf) => pf,
            Err(e) => {
                msg!("the oracle account is not valid: {:?}", e);
                return err!(MarginOrcaErrorCode::InvalidOracle);
            }
        };

    // CHECK: This relies on the margin program verifying oracle staleness.
    // We return the date of the oldest oracle in the pair.
    // TODO: Ensure that this condition is met
    let price_a = token_a_oracle.get_price_unchecked();
    let price_b = token_b_oracle.get_price_unchecked();
    let price_a_num = Number128::from_decimal(price_a.price, price_a.expo);
    let price_b_num = Number128::from_decimal(price_b.price, price_b.expo);

    let pair_price = price_a_num
        .safe_div(price_b_num)?
        .safe_mul(Number128::ONE)?
        .as_f64();

    dbg!((pair_price, price_a_num, price_b_num));

    // In a SOL/USDC pair where SOL decimals = 9 and USDC = 6, expo = 0.001;
    let expo = 10f64.powi(info.mint_b_decimals as i32 - info.mint_a_decimals as i32);
    // Get the approximate tick index which the whirlpool prices should be within to be considered valid.
    // If the price is too far from this tick index, the whirlpool's positions will carry a 0 value
    // until the price stabilises. The user should still be able to close their position by withdrawing
    // tokens, provided that their margin account remains healthy after the withdrawal.
    let approx_tick_index = f64::log(pair_price * expo, 1.0001).round() as i32;
    let (min_tick_index, max_tick_index) = (
        approx_tick_index * (100 - PRICE_TICK_TOLERANCE) / 100,
        approx_tick_index * (100 + PRICE_TICK_TOLERANCE) / 100,
    );

    let mut remaining_accounts = ctx.remaining_accounts.iter();
    // Get whirlpools
    let whirlpools = remaining_accounts
        .by_ref()
        .take(total_whirlpools)
        .map(|account| {
            let whirlpool = Account::<orca_whirlpool::state::Whirlpool>::try_from(account)
                .unwrap()
                .into_inner();
            (
                account.key(),
                (whirlpool.tick_current_index, whirlpool.sqrt_price),
            )
        })
        .collect::<HashMap<_, _>>();

    // Collect positions from remaining_accounts
    // Collecting the positions into a hashmap prevents duplicate positions,
    // as otherwise a user could duplicate positions and trick the margin program into recording
    // twice the value.
    let positions = HashMap::from_iter(remaining_accounts.filter_map(|account| {
        // TODO: we can handle this error better
        let position = Account::<Position>::try_from(account).unwrap().into_inner();
        let whirlpool = whirlpools.get(&position.whirlpool).unwrap();

        // TODO: will remove the dbg after we review this.
        dbg!((min_tick_index, whirlpool.0, max_tick_index));

        if whirlpool.0 < min_tick_index || whirlpool.0 > max_tick_index {
            msg!("Whirlpool price is too far from the current price");
            return None;
        }

        Some((
            account.key(),
            (PositionValuation {
                position,
                current_tick_index: whirlpool.0,
                sqrt_price: whirlpool.1,
            }),
        ))
    }));

    // TODO: all positions should be supplied

    let (token_balance_a, token_balance_b) = meta.position_token_balances(&positions)?;

    // // We don't yet have logic to handle oracles with different exponents.
    // // We expect Pyth to use the same exponents, however let's check that
    // assert_eq!(price_a.expo, price_b.expo);

    // Calculate the weighted value of both tokens
    let balance_a = Number128::from_decimal(token_balance_a, -(info.mint_a_decimals as i32));
    let balance_b = Number128::from_decimal(token_balance_b, -(info.mint_b_decimals as i32));

    let value_a = balance_a.safe_mul(price_a_num)?;
    let value_b = balance_b.safe_mul(price_b_num)?;
    let total_value = value_a + value_b;
    // We divide by 1 to prevent an overflow issue
    let unit_value = total_value.safe_div(Number128::ONE)?;

    let unit_value_i64: i64 =
        i64::try_from(unit_value.as_u64(POSITION_VALUE_EXPO)).expect("Value overflowed");

    // Tell the margin program what the current prices are
    jet_margin::write_adapter_result(
        &*ctx.accounts.owner.load()?,
        &AdapterResult {
            position_changes: vec![(
                info.position_mint,
                vec![PositionChange::Price(PriceChangeInfo {
                    publish_time: price_a.publish_time.min(price_b.publish_time),
                    exponent: POSITION_VALUE_EXPO,
                    value: unit_value_i64,
                    confidence: 0,        // TODO
                    twap: unit_value_i64, // TODO
                })],
            )],
        },
    )?;

    Ok(())
}
