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
use anchor_spl::token;
use solana_program::clock::UnixTimestamp;

use crate::{
    syscall::{sys, Sys},
    ErrorCode, MarginAccount, PriceChangeInfo, TokenConfig, TokenOracle,
};

#[derive(Accounts)]
pub struct RefreshDepositPosition<'info> {
    /// The account to update
    #[account(mut)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The margin config for the token
    #[account(constraint = config.airspace == margin_account.load()?.airspace @ ErrorCode::WrongAirspace)]
    pub config: Account<'info, TokenConfig>,

    /// The oracle for the token
    pub price_oracle: AccountInfo<'info>,
    // Optional account (remaining accounts)
    // pub position_token_account: Account<'info, TokenAccount>,
}

pub fn refresh_deposit_position_handler(ctx: Context<RefreshDepositPosition>) -> Result<()> {
    let mut margin_account = ctx.accounts.margin_account.load_mut()?;
    let config = &ctx.accounts.config;

    match config.oracle() {
        Some(TokenOracle::Pyth { price, .. }) => {
            let price_oracle_key = ctx.accounts.price_oracle.key();
            if price_oracle_key != price {
                msg!("expected oracle {} but got {}", price, price_oracle_key);
                return err!(ErrorCode::InvalidOracle);
            }

            let price_feed = match pyth_sdk_solana::load_price_feed_from_account_info(
                &ctx.accounts.price_oracle,
            ) {
                Ok(pf) => pf,
                Err(e) => {
                    msg!("the oracle account is not valid: {:?}", e);
                    return err!(ErrorCode::InvalidOracle);
                }
            };

            // Price will be checked by the margin program
            let price_obj = price_feed.get_price_unchecked();
            let ema_obj = price_feed.get_ema_price_unchecked();

            let price_info = PriceChangeInfo {
                value: price_obj.price,
                confidence: price_obj.conf,
                twap: ema_obj.price,
                exponent: price_obj.expo,
                publish_time: price_obj.publish_time,
            };

            if let Some(position_token_account) = ctx.remaining_accounts.first() {
                let balance = token::accessor::amount(position_token_account)?;

                margin_account.set_position_balance(
                    &config.mint,
                    &position_token_account.key(),
                    balance,
                    sys().unix_timestamp(),
                )?;
            }

            margin_account.set_position_price(
                &config.mint,
                &price_info.try_into(sys().unix_timestamp() as UnixTimestamp)?,
            )?;
        }

        None => {
            return err!(ErrorCode::InvalidOracle);
        }
    }

    Ok(())
}
