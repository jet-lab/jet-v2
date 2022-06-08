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
use anchor_spl::token::{self, CloseAccount, Mint, Token, TokenAccount};

use crate::{MarginAccount, SignerSeeds};

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    /// The authority that can change the margin account
    pub authority: Signer<'info>,

    /// The receiver for the rent released
    /// CHECK:
    #[account(mut)]
    pub receiver: AccountInfo<'info>,

    /// The margin account with the position to close
    #[account(mut, constraint = margin_account.load().unwrap().has_authority(authority.key()))]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The mint for the position token being deregistered
    pub position_token_mint: Account<'info, Mint>,

    /// The token account for the position being closed
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> ClosePosition<'info> {
    fn close_token_account_ctx(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            CloseAccount {
                account: self.token_account.to_account_info(),
                authority: self.margin_account.to_account_info(),
                destination: self.receiver.to_account_info(),
            },
        )
    }
}

pub fn close_position_handler(ctx: Context<ClosePosition>) -> Result<()> {
    ctx.accounts
        .margin_account
        .load_mut()?
        .unregister_position(
            &ctx.accounts.position_token_mint.key(),
            &ctx.accounts.token_account.key(),
        )?;

    let account = ctx.accounts.margin_account.load()?;
    token::close_account(
        ctx.accounts
            .close_token_account_ctx()
            .with_signer(&[&account.signer_seeds()]),
    )?;

    Ok(())
}
