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

use jet_metadata::PositionTokenMetadata;

use crate::MarginAccount;

#[derive(Accounts)]
pub struct RegisterPosition<'info> {
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
    #[account(has_one = position_token_mint)]
    pub metadata: Account<'info, PositionTokenMetadata>,

    /// The token account to store hold the position assets in the custody of the
    /// margin account.
    #[account(init,
              seeds = [
                  margin_account.key().as_ref(),
                  position_token_mint.key().as_ref()
              ],
              bump,
              payer = payer,
              token::mint = position_token_mint,
              token::authority = margin_account
    )]
    pub token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn register_position_handler(ctx: Context<RegisterPosition>) -> Result<()> {
    let metadata = &ctx.accounts.metadata;
    let mut account = ctx.accounts.margin_account.load_mut()?;
    let position_token = &ctx.accounts.position_token_mint;
    let address = ctx.accounts.token_account.key();

    account.register_position(
        position_token.key(),
        position_token.decimals,
        address,
        metadata.adapter_program,
        metadata.token_kind.into(),
        metadata.collateral_weight,
        metadata.collateral_max_staleness,
    )?;

    Ok(())
}
