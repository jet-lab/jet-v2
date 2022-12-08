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
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

use crate::{error::TestServiceError, seeds::TOKEN_INFO, state::TokenInfo};

#[derive(Accounts)]
pub struct TokenRequest<'info> {
    /// user requesting tokens
    #[account(mut)]
    pub requester: Signer<'info>,

    /// The relevant token mint
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// The test info for the token
    #[account(has_one = mint)]
    pub info: Account<'info, TokenInfo>,

    /// The destination account for the minted tokens
    #[account(mut)]
    pub destination: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn token_request_handler(ctx: Context<TokenRequest>, amount: u64) -> Result<()> {
    let info = &mut ctx.accounts.info;

    // Check requested amount against limit if the requester is not the authority
    if info.authority != ctx.accounts.requester.key() && amount > info.max_request_amount {
        return err!(TestServiceError::PermissionDenied);
    }

    let bump_seed = info.bump_seed;

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
                authority: ctx.accounts.info.to_account_info(),
            },
            &[&[TOKEN_INFO, ctx.accounts.mint.key().as_ref(), &[bump_seed]]],
        ),
        amount,
    )?;

    Ok(())
}
