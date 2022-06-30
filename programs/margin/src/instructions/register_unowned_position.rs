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
use anchor_spl::token::{Mint, Token, TokenAccount};

use jet_metadata::{PositionTokenMetadata, TokenKind};

use crate::{events, ErrorCode, MarginAccount};

#[derive(Accounts)]
pub struct RegisterUnownedPosition<'info> {
    /// The authority that can change the margin account
    pub authority: Signer<'info>,

    /// The address paying for rent
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The margin account to register position type with
    #[account(mut, constraint = margin_account.load().unwrap().has_authority(authority.key()))]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The mint for the position token being registered
    pub position_token_mint: Account<'info, Mint>,

    /// The metadata account that references the correct oracle for the token
    #[account(
        has_one = position_token_mint,
        constraint = metadata.token_kind == TokenKind::Claim @ ErrorCode::InvalidPositionOwner,
    )]
    pub metadata: Account<'info, PositionTokenMetadata>,

    /// The token account that custodies the position assets for the margin account.
    #[account(
        constraint = token_account.owner == metadata.owner    @ ErrorCode::InvalidPositionOwner,
        constraint = token_account.owner != Pubkey::default() @ ErrorCode::InvalidPositionOwner,
    )]
    pub token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn register_unowned_position_handler(ctx: Context<RegisterUnownedPosition>) -> Result<()> {
    let metadata = &ctx.accounts.metadata;
    let mut account = ctx.accounts.margin_account.load_mut()?;
    let position_token = &ctx.accounts.position_token_mint;
    let address = ctx.accounts.token_account.key();

    let position = account.register_position(
        position_token.key(),
        position_token.decimals,
        address,
        metadata.adapter_program,
        metadata.token_kind.into(),
        metadata.value_modifier,
        metadata.max_staleness,
    )?;

    emit!(events::PositionRegistered {
        margin_account: ctx.accounts.margin_account.key(),
        authority: ctx.accounts.authority.key(),
        position,
    });

    Ok(())
}
