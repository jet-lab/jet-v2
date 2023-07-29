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

use anchor_spl::token::{burn, Burn, Mint, Token, TokenAccount};
use orca_whirlpool::program::Whirlpool;

use crate::*;

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(signer)]
    pub owner: AccountLoader<'info, MarginAccount>,

    #[account(mut)]
    pub receiver: Signer<'info>,

    #[account(mut)]
    pub whirlpool_config: Box<Account<'info, WhirlpoolConfig>>,

    #[account(mut, has_one = owner)]
    pub adapter_position_metadata: Box<Account<'info, PositionMetadata>>,

    /// CHECK: will be validated by orca
    #[account(mut,
        seeds = [b"position", position_mint.key().as_ref()],
        seeds::program = Whirlpool::id(),
        bump
    )]
    pub position: UncheckedAccount<'info>,

    /// CHECK: will be validated by orca
    #[account(mut)]
    pub position_mint: UncheckedAccount<'info>,

    /// CHECK: will be validated by orca
    #[account(mut)]
    pub position_token_account: UncheckedAccount<'info>,

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
        seeds = [
            seeds::POSITION_NOTES,
            whirlpool_config.key().as_ref(),
        ],
        bump,
        mint::authority = whirlpool_config
    )]
    pub margin_position_mint: Box<Account<'info, Mint>>,

    pub orca_program: Program<'info, Whirlpool>,
    pub token_program: Program<'info, Token>,
}

impl<'info> ClosePosition<'info> {
    #[inline(never)]
    pub fn close_position(&self) -> Result<()> {
        orca_whirlpool::cpi::close_position(CpiContext::new(
            self.orca_program.to_account_info(),
            orca_whirlpool::cpi::accounts::ClosePosition {
                position: self.position.to_account_info(),
                position_mint: self.position_mint.to_account_info(),
                position_token_account: self.position_token_account.to_account_info(),
                token_program: self.token_program.to_account_info(),
                position_authority: self.owner.to_account_info(),
                receiver: self.receiver.to_account_info(),
            },
        ))?;

        Ok(())
    }
}

pub fn close_position_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, ClosePosition<'info>>,
) -> Result<()> {
    let position_address = ctx.accounts.position.key();
    let position_index = ctx
        .accounts
        .adapter_position_metadata
        .position_index(position_address)
        .unwrap();

    ctx.accounts.close_position()?;
    ctx.accounts
        .adapter_position_metadata
        .clear_position(position_index)?;

    // Increment the global number of positions owned
    ctx.accounts.whirlpool_config.total_positions = ctx
        .accounts
        .whirlpool_config
        .total_positions
        .checked_sub(1)
        .ok_or(MarginOrcaErrorCode::ArithmeticError)?;

    // Burn position note
    burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.margin_position_mint.to_account_info(),
                from: ctx.accounts.margin_position.to_account_info(),
                authority: ctx.accounts.whirlpool_config.to_account_info(),
            },
        )
        .with_signer(&[&ctx.accounts.whirlpool_config.authority_seeds()]),
        1,
    )?;

    // TODO: do we need to update margin?

    Ok(())
}
