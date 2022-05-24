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
#[cfg(not(feature = "devnet"))]
use anchor_lang::solana_program::pubkey;

use jet_margin_pool::MarginPoolConfig;

mod instructions;
use instructions::*;

pub use instructions::{MarginPoolParams, TokenMetadataParams};

declare_id!("JPCtrLreUqsEbdhtxZ8zpd8wBydKz4nuEjX5u9Eg5H8");

#[cfg(not(feature = "devnet"))]
static ROOT_AUTHORITY: Pubkey = pubkey!("FqXoGb9Zxy4uzG12N1jvHyktNG3Zsez367vAzJeiyMF1");

#[program]
mod jet_control {

    use super::*;

    /// Create the master authority account
    pub fn create_authority(ctx: Context<CreateAuthority>) -> Result<()> {
        instructions::create_authority_handler(ctx)
    }

    /// Register an SPL token for use with the protocol, by creating
    /// a margin pool which can accept deposits for the token.
    ///
    /// Does not require special permission
    pub fn register_token(ctx: Context<RegisterToken>) -> Result<()> {
        instructions::register_token_handler(ctx)
    }

    /// Register a program to be allowed for use as margin adapter in the
    /// protocol.
    pub fn register_adapter(ctx: Context<RegisterAdapter>) -> Result<()> {
        instructions::register_adapter_handler(ctx)
    }

    /// Configure details about a token
    pub fn configure_token(
        ctx: Context<ConfigureToken>,
        metadata: Option<TokenMetadataParams>,
        pool_param: Option<MarginPoolParams>,
        pool_config: Option<MarginPoolConfig>,
    ) -> Result<()> {
        instructions::configure_token_handler(ctx, metadata, pool_param, pool_config)
    }

    /// Configure an address as being allowed to perform the functions
    /// of a liquidator.
    pub fn set_liquidator(ctx: Context<SetLiquidator>, is_liquidator: bool) -> Result<()> {
        instructions::set_liquidator_handler(ctx, is_liquidator)
    }
}
