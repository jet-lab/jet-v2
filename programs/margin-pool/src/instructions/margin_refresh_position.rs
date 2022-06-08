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

use jet_margin::{AdapterResult, MarginAccount, PositionChange, PriceChangeInfo};

use crate::state::*;
use crate::ErrorCode;

#[derive(Accounts)]
pub struct MarginRefreshPosition<'info> {
    /// The margin account being executed on
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The pool to be refreshed
    #[account(has_one = token_price_oracle)]
    pub margin_pool: Account<'info, MarginPool>,

    /// The pyth price account for the pool's token
    /// CHECK:
    pub token_price_oracle: AccountInfo<'info>,
}

pub fn margin_refresh_position_handler(ctx: Context<MarginRefreshPosition>) -> Result<()> {
    let pool = &ctx.accounts.margin_pool;

    // read from the pyth oracle
    let token_oracle = match pyth_sdk_solana::load_price_feed_from_account_info(
        &ctx.accounts.token_price_oracle,
    ) {
        Ok(pf) => pf,
        Err(_) => {
            msg!("the oracle account is not valid");
            return err!(ErrorCode::InvalidOracle);
        }
    };

    let prices = pool.calculate_prices(&token_oracle)?;

    // Tell the margin program what the current prices are
    jet_margin::write_adapter_result(&AdapterResult {
        position_changes: vec![
            (
                pool.deposit_note_mint,
                vec![PositionChange::Price(PriceChangeInfo {
                    publish_time: token_oracle.publish_time,
                    exponent: token_oracle.expo,
                    value: prices.deposit_note_price,
                    confidence: prices.deposit_note_conf,
                    twap: prices.deposit_note_twap,
                })],
            ),
            (
                pool.loan_note_mint,
                vec![PositionChange::Price(PriceChangeInfo {
                    publish_time: token_oracle.publish_time,
                    exponent: token_oracle.expo,
                    value: prices.loan_note_price,
                    confidence: prices.loan_note_conf,
                    twap: prices.loan_note_twap,
                })],
            ),
        ],
    })?;

    Ok(())
}
