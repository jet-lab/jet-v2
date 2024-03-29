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

use jet_margin_pool::MarginPoolConfig;

mod instructions;
use instructions::*;

pub use instructions::TokenMetadataParams;
pub mod events;

declare_id!("JPCtrLreUqsEbdhtxZ8zpd8wBydKz4nuEjX5u9Eg5H8");

pub mod seeds {
    use super::constant;

    #[constant]
    pub const FEE_DESTINATION: &[u8] = b"margin-pool-fee-destination";
}

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
    pub fn create_margin_pool(ctx: Context<CreateMarginPool>) -> Result<()> {
        instructions::create_margin_pool_handler(ctx)
    }

    /// Configure details about a margin pool
    pub fn configure_margin_pool(
        ctx: Context<ConfigureMarginPool>,
        metadata: Option<TokenMetadataParams>,
        pool_config: Option<MarginPoolConfig>,
    ) -> Result<()> {
        instructions::configure_margin_pool_handler(ctx, metadata, pool_config)
    }
}
