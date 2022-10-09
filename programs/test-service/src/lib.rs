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

pub mod error;
pub mod state;

mod instructions;
mod util;

use instructions::*;

pub use instructions::TokenCreateParams;

declare_id!("JPTSApMSqCHBww7vDhpaSmzipTV3qPg6vxub4qneKoy");

pub mod seeds {
    use super::*;

    #[constant]
    pub const TOKEN_MINT: &[u8] = b"token-mint";

    #[constant]
    pub const TOKEN_INFO: &[u8] = b"token-info";

    #[constant]
    pub const TOKEN_PYTH_PRICE: &[u8] = b"token-pyth-price";

    #[constant]
    pub const TOKEN_PYTH_PRODUCT: &[u8] = b"token-pyth-product";
}

#[program]
pub mod jet_test_service {
    use super::*;

    /// Create a token mint based on some seed
    ///
    /// The created mint has a this program as the authority, any user may request
    /// tokens via the `token_request` instruction up to the limit specified in the
    /// `max_amount` parameter.
    ///
    /// This will also create pyth oracle accounts for the token.
    pub fn token_create(ctx: Context<TokenCreate>, params: TokenCreateParams) -> Result<()> {
        token_create_handler(ctx, params)
    }

    /// Initialize the token info and oracles for the native token mint
    ///
    /// Since the native mint is a special case that can't be owned by this program,
    /// this instruction allows creating an oracle for it.
    pub fn token_init_native(
        ctx: Context<TokenInitNative>,
        oracle_authority: Pubkey,
    ) -> Result<()> {
        token_init_native_handler(ctx, oracle_authority)
    }

    /// Request tokens be minted by the faucet.
    pub fn token_request(ctx: Context<TokenRequest>, amount: u64) -> Result<()> {
        token_request_handler(ctx, amount)
    }

    /// Update the pyth oracle price account for a token
    pub fn token_update_pyth_price(
        ctx: Context<TokenUpdatePythPrice>,
        price: i64,
        conf: i64,
        expo: i32,
    ) -> Result<()> {
        token_update_pyth_price_handler(ctx, price, conf, expo)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
