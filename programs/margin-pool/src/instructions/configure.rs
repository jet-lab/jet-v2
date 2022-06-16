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

use jet_metadata::ControlAuthority;

use crate::state::*;
use crate::ErrorCode;

#[derive(Accounts)]
pub struct Configure<'info> {
    /// The pool to be configured
    #[account(mut)]
    pub margin_pool: Account<'info, MarginPool>,

    /// The authority allowed to modify the pool, which must sign
    #[cfg_attr(not(feature = "testing"), account(signer))]
    pub authority: Account<'info, ControlAuthority>,

    /// CHECK:
    pub pyth_product: AccountInfo<'info>,

    /// CHECK:
    pub pyth_price: AccountInfo<'info>,
}

pub fn configure_handler(
    ctx: Context<Configure>,
    fee_destination: Option<Pubkey>,
    config: Option<MarginPoolConfig>,
) -> Result<()> {
    let pool = &mut ctx.accounts.margin_pool;

    if let Some(new_fee_destination) = fee_destination {
        pool.fee_destination = new_fee_destination;
    }

    if let Some(new_config) = config {
        pool.config = new_config;
    }

    if *ctx.accounts.pyth_price.key != Pubkey::default() {
        let product_data = ctx.accounts.pyth_product.try_borrow_data()?;
        let product_account = pyth_sdk_solana::state::load_product_account(&**product_data)
            .map_err(|_| ErrorCode::InvalidOracle)?;

        let expected_price_key = Pubkey::new_from_array(product_account.px_acc.val);
        if expected_price_key != *ctx.accounts.pyth_price.key {
            msg!("oracle product account does not match price account");
            return err!(ErrorCode::InvalidOracle);
        }

        //TODO JV2M-359
        //TODO this needs to be set in the product account.
        let quote_currency = product_account
            .iter()
            .find_map(|(k, v)| match k {
                "quote_currency" => Some(v),
                _ => None,
            })
            .expect("product has no quote_currency");

        if quote_currency != "USD" {
            msg!("this oracle does not quote prices in USD");
            return err!(ErrorCode::InvalidOracle);
        }

        pool.token_price_oracle = ctx.accounts.pyth_price.key();
        msg!("oracle = {}", &pool.token_price_oracle);
    }

    Ok(())
}
