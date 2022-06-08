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

use jet_margin_pool::program::JetMarginPool;
use jet_margin_pool::MarginPoolConfig;
use jet_margin_pool::{cpi::accounts::Configure, MarginPool};
use jet_metadata::cpi::accounts::SetEntry;
use jet_metadata::program::JetMetadata;
use jet_metadata::{PositionTokenMetadata, TokenKind, TokenMetadata};

use super::Authority;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TokenMetadataParams {
    /// Description of this token
    pub token_kind: TokenKind,

    /// The weight of the asset's value relative to other tokens when used as collateral.
    pub collateral_weight: u16,

    /// The maximum staleness (seconds) that's acceptable for this token when used as collateral.
    pub collateral_max_staleness: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MarginPoolParams {
    pub fee_destination: Pubkey,
}

#[derive(Accounts)]
pub struct ConfigureToken<'info> {
    #[cfg_attr(not(feature = "devnet"), account(address = crate::ROOT_AUTHORITY))]
    pub requester: Signer<'info>,
    pub authority: Account<'info, Authority>,

    /// CHECK:
    pub token_mint: UncheckedAccount<'info>,

    #[account(mut, has_one = token_mint)]
    pub margin_pool: Account<'info, MarginPool>,

    #[account(mut, has_one = token_mint)]
    pub token_metadata: Account<'info, TokenMetadata>,

    #[account(mut, constraint = deposit_metadata.underlying_token_mint == token_mint.key())]
    pub deposit_metadata: Account<'info, PositionTokenMetadata>,

    /// CHECK:
    pub pyth_product: UncheckedAccount<'info>,

    /// CHECK:
    pub pyth_price: UncheckedAccount<'info>,

    pub margin_pool_program: Program<'info, JetMarginPool>,
    pub metadata_program: Program<'info, JetMetadata>,
}

impl<'info> ConfigureToken<'info> {
    fn configure_pool_context(&self) -> CpiContext<'_, '_, '_, 'info, Configure<'info>> {
        CpiContext::new(
            self.margin_pool_program.to_account_info(),
            Configure {
                margin_pool: self.margin_pool.to_account_info(),
                authority: self.authority.to_account_info(),
                pyth_product: self.pyth_product.to_account_info(),
                pyth_price: self.pyth_price.to_account_info(),
            },
        )
    }

    fn set_metadata_context(&self) -> CpiContext<'_, '_, '_, 'info, SetEntry<'info>> {
        CpiContext::new(
            self.metadata_program.to_account_info(),
            SetEntry {
                metadata_account: self.token_metadata.to_account_info(),
                authority: self.authority.to_account_info(),
            },
        )
    }

    fn set_deposit_metadata_context(&self) -> CpiContext<'_, '_, '_, 'info, SetEntry<'info>> {
        CpiContext::new(
            self.metadata_program.to_account_info(),
            SetEntry {
                metadata_account: self.deposit_metadata.to_account_info(),
                authority: self.authority.to_account_info(),
            },
        )
    }
}

#[inline(never)]
pub fn configure_token_handler(
    ctx: Context<ConfigureToken>,
    metadata: Option<TokenMetadataParams>,
    pool_param: Option<MarginPoolParams>,
    pool_config: Option<MarginPoolConfig>,
) -> Result<()> {
    let authority = [&ctx.accounts.authority.seed[..]];

    if *ctx.accounts.pyth_price.key != Pubkey::default()
        || pool_param.is_some()
        || pool_config.is_some()
    {
        let fee_destination = pool_param.map(|p| p.fee_destination);

        jet_margin_pool::cpi::configure(
            ctx.accounts
                .configure_pool_context()
                .with_signer(&[&authority]),
            fee_destination,
            pool_config,
        )?;
    }

    if *ctx.accounts.pyth_price.key != Pubkey::default() {
        let mut metadata = ctx.accounts.token_metadata.clone();
        let mut data = vec![];

        metadata.pyth_product = ctx.accounts.pyth_product.key();
        metadata.pyth_price = ctx.accounts.pyth_price.key();

        metadata.try_serialize(&mut data)?;

        jet_metadata::cpi::set_entry(
            ctx.accounts
                .set_metadata_context()
                .with_signer(&[&authority]),
            0,
            data,
        )?;
    }

    if let Some(params) = metadata {
        let mut metadata = ctx.accounts.deposit_metadata.clone();
        let mut data = vec![];

        metadata.token_kind = params.token_kind;
        metadata.collateral_weight = params.collateral_weight;
        metadata.collateral_max_staleness = params.collateral_max_staleness;

        metadata.try_serialize(&mut data)?;

        jet_metadata::cpi::set_entry(
            ctx.accounts
                .set_deposit_metadata_context()
                .with_signer(&[&authority]),
            0,
            data,
        )?;
    }

    Ok(())
}
