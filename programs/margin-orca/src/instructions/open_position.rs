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

use std::collections::BTreeMap;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};
use orca_whirlpool::{program::Whirlpool, state::OpenPositionBumps};

use crate::*;

#[derive(Accounts)]
#[instruction(bumps: OpenPositionBumps, seed: u64)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(signer)]
    pub owner: AccountLoader<'info, MarginAccount>,

    #[account(mut)]
    pub whirlpool_config: Box<Account<'info, WhirlpoolConfig>>,

    /// CHECK: will be initialized and validated by orca
    #[account(mut,
        seeds = [b"position", position_mint.key().as_ref()],
        seeds::program = Whirlpool::id(),
        bump = bumps.position_bump
    )]
    pub position: UncheckedAccount<'info>,

    /// CHECK: will be initialized and validated by orca
    #[account(mut,
        seeds = [
            seeds::POSITION_MINT,
            seed.to_le_bytes().as_ref(),
            owner.key().as_ref(),
            whirlpool.key().as_ref()
        ],
        bump
    )]
    pub position_mint: AccountInfo<'info>,

    /// CHECK: will be initialized and validated by orca
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

    #[account(mut, has_one = owner, has_one = whirlpool_config)]
    pub adapter_position_metadata: Box<Account<'info, PositionMetadata>>,

    pub whirlpool: Box<Account<'info, orca_whirlpool::state::Whirlpool>>,

    pub orca_program: Program<'info, Whirlpool>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> OpenPosition<'info> {
    /// Issue a Orca swap
    #[allow(clippy::too_many_arguments)]
    #[inline(never)]
    pub fn open_position(
        &self,
        bumps: OpenPositionBumps,
        ctx_bumps: &BTreeMap<String, u8>,
        seed: u64,
        tick_lower_index: i32,
        tick_upper_index: i32,
    ) -> Result<()> {
        orca_whirlpool::cpi::open_position(
            CpiContext::new_with_signer(
                self.orca_program.to_account_info(),
                orca_whirlpool::cpi::accounts::OpenPosition {
                    funder: self.payer.to_account_info(),
                    owner: self.owner.to_account_info(),
                    position: self.position.to_account_info(),
                    position_mint: self.position_mint.to_account_info(),
                    position_token_account: self.position_token_account.to_account_info(),
                    whirlpool: self.whirlpool.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    rent: self.rent.to_account_info(),
                    associated_token_program: self.associated_token_program.to_account_info(),
                },
                &[&[
                    seeds::POSITION_MINT,
                    seed.to_le_bytes().as_ref(),
                    self.owner.key().as_ref(),
                    self.whirlpool.key().as_ref(),
                    &[*ctx_bumps.get("position_mint").unwrap()],
                ]],
            ),
            bumps,
            tick_lower_index,
            tick_upper_index,
        )?;

        Ok(())
    }
}

pub fn open_position_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, OpenPosition<'info>>,
    bumps: OpenPositionBumps,
    seed: u64,
    tick_lower_index: i32,
    tick_upper_index: i32,
) -> Result<()> {
    // Check if there is enough space for the position
    let empty_position_ix = ctx
        .accounts
        .adapter_position_metadata
        .free_position()
        .ok_or(MarginOrcaErrorCode::PositionsFull)?;

    ctx.accounts
        .open_position(bumps, &ctx.bumps, seed, tick_lower_index, tick_upper_index)?;
    // Mint a position note
    mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.margin_position_mint.to_account_info(),
                to: ctx.accounts.margin_position.to_account_info(),
                authority: ctx.accounts.whirlpool_config.to_account_info(),
            },
        )
        .with_signer(&[&ctx.accounts.whirlpool_config.authority_seeds()]),
        1,
    )?;

    // Increment the global number of positions owned
    ctx.accounts.whirlpool_config.total_positions = ctx
        .accounts
        .whirlpool_config
        .total_positions
        .checked_add(1)
        .ok_or(MarginOrcaErrorCode::ArithmeticError)?;

    // Update position metadata
    ctx.accounts.adapter_position_metadata.set_position(
        ctx.accounts.position.key(),
        ctx.accounts.whirlpool.key(),
        empty_position_ix,
    )?;

    Ok(())
}
