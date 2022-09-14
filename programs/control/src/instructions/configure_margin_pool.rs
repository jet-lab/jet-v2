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

use jet_margin::cpi::accounts::{MutateToken, PositionTokenAccounts};
use jet_margin::program::JetMargin;
use jet_margin::{PositionKind, PositionParams, TokenMeta};
use jet_margin_pool::program::JetMarginPool;
use jet_margin_pool::MarginPoolConfig;
use jet_margin_pool::{cpi::accounts::Configure, MarginPool};
use jet_metadata::TokenKind;

use crate::events;

use super::Authority;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct TokenMetadataParams {
    /// Description of this token
    pub token_kind: TokenKind,

    /// The weight of the asset's value relative to other tokens when used as collateral.
    pub collateral_weight: u16,

    /// The maximum leverage allowed on loans for the token
    pub max_leverage: u16,
}

#[derive(Accounts)]
pub struct ConfigureMarginPool<'info> {
    #[cfg_attr(not(feature = "testing"), account(address = crate::ROOT_AUTHORITY))]
    pub requester: Signer<'info>,
    pub authority: Box<Account<'info, Authority>>,
    pub token_mint: UncheckedAccount<'info>,

    #[account(mut, has_one = token_mint)]
    pub margin_pool: Box<Account<'info, MarginPool>>,

    #[account(mut, has_one = token_mint)]
    pub token_metadata: Box<Account<'info, TokenMeta>>,

    #[account(mut, constraint = deposit_metadata.underlying_mint == token_mint.key())]
    pub deposit_metadata: Box<Account<'info, TokenMeta>>,

    #[account(mut, constraint = loan_metadata.underlying_mint == token_mint.key())]
    pub loan_metadata: Box<Account<'info, TokenMeta>>,

    pub pyth_product: UncheckedAccount<'info>,
    pub pyth_price: UncheckedAccount<'info>,
    pub margin_pool_program: Program<'info, JetMarginPool>,
    pub margin_program: Program<'info, JetMargin>,
    pub system_program: Program<'info, System>,
}

impl<'info> ConfigureMarginPool<'info> {
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

    fn set_deposit_metadata_context(&self) -> CpiContext<'_, '_, '_, 'info, MutateToken<'info>> {
        CpiContext::new(
            self.margin_program.to_account_info(),
            MutateToken {
                metadata: self.deposit_metadata.to_account_info(),
                other: PositionTokenAccounts {
                    requester: self.requester.to_account_info(),
                    token_mint: self.system_program.to_account_info(),
                    adapter_program: self.margin_pool_program.to_account_info(),
                    pyth_price: self.pyth_price.to_account_info(),
                    pyth_product: self.pyth_product.to_account_info(),
                },
            },
        )
    }

    fn set_loan_metadata_context(&self) -> CpiContext<'_, '_, '_, 'info, MutateToken<'info>> {
        CpiContext::new(
            self.margin_program.to_account_info(),
            MutateToken {
                metadata: self.loan_metadata.to_account_info(),
                other: PositionTokenAccounts {
                    requester: self.requester.to_account_info(),
                    token_mint: self.system_program.to_account_info(),
                    adapter_program: self.margin_pool_program.to_account_info(),
                    pyth_price: self.pyth_price.to_account_info(),
                    pyth_product: self.pyth_product.to_account_info(),
                },
            },
        )
    }
}

#[inline(never)]
pub fn configure_margin_pool_handler(
    ctx: Context<ConfigureMarginPool>,
    metadata: Option<TokenMetadataParams>,
    pool_config: Option<MarginPoolConfig>,
) -> Result<()> {
    let authority = [&ctx.accounts.authority.seed[..]];

    if *ctx.accounts.pyth_price.key != Pubkey::default() || pool_config.is_some() {
        jet_margin_pool::cpi::configure(
            ctx.accounts
                .configure_pool_context()
                .with_signer(&[&authority]),
            pool_config,
        )?;
    }

    jet_margin::cpi::mutate_token(
        ctx.accounts
            .set_deposit_metadata_context()
            .with_signer(&[&authority]),
        metadata.as_ref().map(|params| PositionParams {
            position_kind: params.token_kind.into(),
            value_modifier: params.collateral_weight,
            max_staleness: 0,
        }),
    )?;
    emit!(events::PositionTokenMetadataConfigured {
        requester: ctx.accounts.requester.key(),
        authority: ctx.accounts.authority.key(),
        metadata_account: ctx.accounts.deposit_metadata.key(),
    });

    jet_margin::cpi::mutate_token(
        ctx.accounts
            .set_loan_metadata_context()
            .with_signer(&[&authority]),
        metadata.as_ref().map(|params| PositionParams {
            position_kind: PositionKind::Claim,
            value_modifier: params.max_leverage,
            max_staleness: 0,
        }),
    )?;
    emit!(events::PositionTokenMetadataConfigured {
        requester: ctx.accounts.requester.key(),
        authority: ctx.accounts.authority.key(),
        metadata_account: ctx.accounts.loan_metadata.key(),
    });

    Ok(())
}
