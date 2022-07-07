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
use anchor_spl::token::{Token, TokenAccount};

use jet_margin::{AdapterResult, MarginAccount, PositionChange};

use crate::state::*;

#[derive(Accounts)]
pub struct RegisterLoan<'info> {
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// This will be required for margin to register the position,
    /// so requiring it here makes it easier for clients to ensure
    /// that it will be sent.
    pub position_token_metadata: AccountInfo<'info>,

    /// The token account to store the loan notes representing the claim
    /// against the margin account
    #[account(init,
        seeds = [margin_account.key().as_ref(),
                 loan_note_mint.key().as_ref()],
        bump,
        payer = payer,
        token::mint = loan_note_mint,
        token::authority = margin_pool
    )]
    pub loan_note_account: Account<'info, TokenAccount>,

    /// The mint for the notes representing loans from the pool
    /// CHECK:
    pub loan_note_mint: AccountInfo<'info>,

    #[account(has_one = loan_note_mint)]
    pub margin_pool: Account<'info, MarginPool>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn register_loan_handler(ctx: Context<RegisterLoan>) -> Result<()> {
    jet_margin::write_adapter_result(
        &*ctx.accounts.margin_account.load()?,
        &AdapterResult {
            position_changes: vec![(
                ctx.accounts.loan_note_mint.key(),
                vec![PositionChange::Register(
                    ctx.accounts.loan_note_account.key(),
                )],
            )],
        },
    )?;

    Ok(())
}
