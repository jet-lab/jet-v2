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

use std::convert::TryInto;

use anchor_lang::prelude::*;

use crate::{util::Require, MarginAccount, PriceChangeInfo, TokenMeta};

#[derive(Accounts)]
pub struct RefreshPositionPrice<'info> {
    /// The margin account with the position to be refreshed
    #[account(mut)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The metadata account for the token, which has been updated
    pub metadata: Account<'info, TokenMeta>,

    // pub pyth_product: AccountInfo<'info>,
    pub pyth_price: AccountInfo<'info>,

    // remaining_accounts: [Account<'info, TokenMeta>] (for underlying)
}

/// Refresh the metadata for a position
pub fn refresh_position_price_handler<'c, 'info>(
    ctx: Context<'_, '_, 'c, 'info, RefreshPositionPrice<'info>>,
) -> Result<()> {
    // read from the pyth oracle
    let price = get_price_from_oracle(
        &ctx.accounts.metadata,
        &ctx.accounts.pyth_price,
        ctx.remaining_accounts.iter(),
    )?;
    // set price for position in margin account
    ctx.accounts
        .margin_account
        .load_mut()?
        .get_position_mut(&ctx.accounts.metadata.token_mint)
        .require()?
        .set_price(&price.try_into()?)?;

    Ok(())
}

/// todo errors
fn get_price_from_oracle<'c, 'info: 'c, I: Iterator<Item = &'c AccountInfo<'info>>>(
    metadata: &Account<TokenMeta>,
    pyth_price_info: &AccountInfo<'info>,
    mut remaining_accounts: I,
) -> Result<PriceChangeInfo> {
    match metadata.price_source() {
        crate::PriceSource::Adapter(_) => panic!("must be priced by adapter"),
        crate::PriceSource::Underlying(underlying) => {
            let underyling_meta =
                Account::<TokenMeta>::try_from(&remaining_accounts.next().unwrap())?;
            if underyling_meta.token_mint != underlying {
                panic!("wrong underlying meta");
            } else {
                get_price_from_oracle(&underyling_meta, pyth_price_info, remaining_accounts)
            }
        }
        crate::PriceSource::Oracle {
            pyth_price,
            pyth_product: _,
        } => {
            assert_eq!(&pyth_price, pyth_price_info.key);
            let price_feed =
                pyth_sdk_solana::load_price_feed_from_account_info(&pyth_price_info).unwrap();
            let price_obj = price_feed.get_current_price().unwrap();
            let ema_obj = price_feed.get_ema_price().unwrap();
            Ok(PriceChangeInfo {
                value: price_obj.price,
                confidence: price_obj.conf,
                twap: ema_obj.price,
                publish_time: price_feed.publish_time,
                exponent: price_feed.expo,
            })
        }
    }
}
