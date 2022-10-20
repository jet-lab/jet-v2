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

use pyth_sdk_solana::state::{PriceAccount, PriceStatus, Rational};

use crate::{state::TokenInfo, util::load_pyth_account};

#[derive(Accounts)]
pub struct TokenUpdatePythPrice<'info> {
    oracle_authority: Signer<'info>,

    #[account(has_one = pyth_price)]
    info: Account<'info, TokenInfo>,

    #[account(mut)]
    pyth_price: AccountInfo<'info>,
}

pub fn token_update_pyth_price_handler(
    ctx: Context<TokenUpdatePythPrice>,
    price: i64,
    conf: i64,
    expo: i32,
) -> Result<()> {
    let mut pyth_price = load_pyth_account::<PriceAccount>(&ctx.accounts.pyth_price)?;
    let clock = Clock::get()?;

    pyth_price.expo = expo;
    pyth_price.agg.price = price;
    pyth_price.agg.conf = conf as u64;
    pyth_price.agg.status = PriceStatus::Trading;
    pyth_price.agg.pub_slot = clock.slot;

    pyth_price.ema_price = Rational {
        val: price,
        numer: price,
        denom: 1,
    };

    pyth_price.ema_conf = Rational {
        val: conf,
        numer: conf,
        denom: 1,
    };

    pyth_price.last_slot = clock.slot;
    pyth_price.valid_slot = clock.slot;
    pyth_price.timestamp = clock.unix_timestamp;

    Ok(())
}
