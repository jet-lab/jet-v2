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

use anchor_spl::token::{close_account, CloseAccount, Mint, Token, TokenAccount};
use jet_margin::{AdapterResult, PositionChange};

use crate::*;

#[derive(Accounts)]
pub struct ClosePositionMeta<'info> {
    #[account(mut)]
    pub receiver: Signer<'info>,

    #[account(signer)]
    pub owner: AccountLoader<'info, MarginAccount>,

    pub whirlpool_config: Box<Account<'info, WhirlpoolConfig>>,

    /// The margin position that tracks the number of open positions
    #[account(mut,
        seeds = [
            seeds::POSITION_NOTES,
            owner.key().as_ref(),
            whirlpool_config.key().as_ref()
        ],
        bump,
        token::mint = margin_position_mint,
        token::authority = whirlpool_config
    )]
    pub margin_position: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        close = receiver,
        has_one = owner
)]
    pub adapter_position_metadata: Box<Account<'info, PositionMetadata>>,

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
}

pub fn close_position_meta_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, ClosePositionMeta<'info>>,
) -> Result<()> {
    if ctx
        .accounts
        .adapter_position_metadata
        .positions()
        .into_iter()
        .count()
        > 0
    {
        return err!(MarginOrcaErrorCode::AccountNotEmpty);
    }

    close_account(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            CloseAccount {
                account: ctx.accounts.margin_position.to_account_info(),
                destination: ctx.accounts.receiver.to_account_info(),
                authority: ctx.accounts.whirlpool_config.to_account_info(),
            },
        )
        .with_signer(&[&ctx.accounts.whirlpool_config.authority_seeds()]),
    )?;

    jet_margin::write_adapter_result(
        &*ctx.accounts.owner.load()?,
        &AdapterResult {
            position_changes: vec![(
                ctx.accounts.margin_position_mint.key(),
                vec![PositionChange::Close(ctx.accounts.margin_position.key())],
            )],
        },
    )?;
    Ok(())
}
