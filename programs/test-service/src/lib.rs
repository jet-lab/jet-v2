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

// Allow this until fixed upstream
#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

pub mod error;
pub mod state;

mod instructions;
mod util;

use instructions::*;

pub use instructions::{SaberSwapPoolCreateParams, SplSwapPoolCreateParams, TokenCreateParams};

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

    #[constant]
    pub const SWAP_POOL_INFO: &[u8] = b"swap-pool-info";

    #[constant]
    pub const SWAP_POOL_STATE: &[u8] = b"swap-pool-state";

    #[constant]
    pub const SWAP_POOL_MINT: &[u8] = b"swap-pool-mint";

    #[constant]
    pub const SWAP_POOL_TOKENS: &[u8] = b"swap-pool-tokens";

    #[constant]
    pub const SWAP_POOL_FEES: &[u8] = b"swap-pool-fees";
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

    /// Same as token_create except it does not create the mint. The mint should
    /// be created some other way, such as by an adapter.
    pub fn token_register(ctx: Context<TokenRegister>, params: TokenCreateParams) -> Result<()> {
        token_register_handler(ctx, params)
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

    /// Create a SPL swap pool
    pub fn spl_swap_pool_create(
        ctx: Context<SplSwapPoolCreate>,
        params: SplSwapPoolCreateParams,
    ) -> Result<()> {
        spl_swap_pool_create_handler(ctx, params)
    }

    /// Balance an SPL swap pool based on current oracle prices
    pub fn spl_swap_pool_balance(ctx: Context<SplSwapPoolBalance>) -> Result<()> {
        spl_swap_pool_balance_handler(ctx)
    }

    /// Invokes arbitrary program iff an account is not yet initialized.
    /// Typically used to run an instruction that initializes the account,
    /// ensuring multiple initializations will not collide.
    /// TODO: this may be generally useful even on mainnet and warrant a separate program
    pub fn if_not_initialized(ctx: Context<IfNotInitialized>, instruction: Vec<u8>) -> Result<()> {
        if_not_initialized_handler(ctx, instruction)
    }

    /// Create a Saber swap pool
    pub fn saber_swap_pool_create(
        ctx: Context<SaberSwapPoolCreate>,
        params: SaberSwapPoolCreateParams,
    ) -> Result<()> {
        saber_swap_pool_create_handler(ctx, params)
    }

    /// Balance an SPL swap pool based on current oracle prices
    pub fn saber_swap_pool_balance(ctx: Context<SaberSwapPoolBalance>) -> Result<()> {
        saber_swap_pool_balance_handler(ctx)
    }
}
