// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright (C) 2023 JET PROTOCOL HOLDINGS, LLC.
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

use anchor_spl::token::{Mint, Token, TokenAccount};
use jet_margin::{AdapterResult, PositionChange};

use crate::*;

#[derive(Accounts)]
pub struct RegisterPositionMeta<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The signing authority of this position meta
    #[account(
        signer,
        constraint = owner.load()?.airspace == whirlpool_config.airspace
    )]
    pub owner: AccountLoader<'info, MarginAccount>,

    #[account(init,
                seeds = [
                    seeds::POSITION_METADATA,
                    owner.key().as_ref(),
                    whirlpool_config.key().as_ref(),
                ],
                bump,
                payer = payer,
                space = PositionMetadata::SIZE
    )]
    pub adapter_position_metadata: Box<Account<'info, PositionMetadata>>,

    pub whirlpool_config: Box<Account<'info, WhirlpoolConfig>>,

    /// This will be required for margin to register the position,
    /// so requiring it here makes it easier for clients to ensure
    /// that it will be sent.
    ///
    /// CHECK: Margin program checks it
    pub position_token_config: AccountInfo<'info>,

    /// Token account used by the margin program to track whirlpools
    #[account(init,
        seeds = [
            seeds::POSITION_NOTES,
            owner.key().as_ref(),
            whirlpool_config.key().as_ref(),
        ],
        bump,
        token::mint = margin_position_mint,
        token::authority = whirlpool_config,
        payer = payer
    )]
    pub margin_position: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [
            seeds::POSITION_NOTES,
            whirlpool_config.key().as_ref(),
        ],
        bump,
        mint::authority = whirlpool_config
    )]
    pub margin_position_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn register_position_meta_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, RegisterPositionMeta<'info>>,
) -> Result<()> {
    // Check that the mint and config match
    if ctx.accounts.whirlpool_config.position_mint != ctx.accounts.margin_position_mint.key() {
        return err!(MarginOrcaErrorCode::InvalidArgument);
    }

    // TODO: when adding config flags, check if new positions can be registered

    let meta = &mut ctx.accounts.adapter_position_metadata;
    meta.owner = ctx.accounts.owner.key();
    meta.whirlpool_config = ctx.accounts.whirlpool_config.key();

    jet_margin::write_adapter_result(
        &*ctx.accounts.owner.load()?,
        &AdapterResult {
            position_changes: vec![(
                ctx.accounts.margin_position_mint.key(),
                vec![PositionChange::Register(ctx.accounts.margin_position.key())],
            )],
        },
    )?;

    Ok(())
}
