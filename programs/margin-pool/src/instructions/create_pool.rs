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

use jet_metadata::ControlAuthority;

use crate::state::*;

#[derive(Accounts)]
pub struct CreatePool<'info> {
    /// The pool to be created
    #[account(
        init,
        seeds = [token_mint.key().as_ref()],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<MarginPool>(),
    )]
    pub margin_pool: Box<Account<'info, MarginPool>>,

    /// The token account holding the pool's deposited funds
    #[account(init,
              seeds = [
                margin_pool.key().as_ref(),
                b"vault".as_ref()
              ],
              bump,
              token::mint = token_mint,
              token::authority = margin_pool,
              payer = payer)]
    pub vault: Box<Account<'info, TokenAccount>>,

    /// The mint for deposit notes
    #[account(init,
              seeds = [
                margin_pool.key().as_ref(),
                b"deposit-notes".as_ref()
              ],
              bump,
              mint::decimals = token_mint.decimals,
              mint::authority = margin_pool,
              payer = payer)]
    pub deposit_note_mint: Box<Account<'info, Mint>>,

    /// The mint for loan notes
    #[account(init,
              seeds = [
                margin_pool.key().as_ref(),
                b"loan-notes".as_ref()
              ],
              bump,
              mint::decimals = token_mint.decimals,
              mint::authority = margin_pool,
              payer = payer)]
    pub loan_note_mint: Box<Account<'info, Mint>>,

    /// The mint for the token being custodied by the pool
    pub token_mint: Box<Account<'info, Mint>>,

    /// The authority to create pools, which must sign
    #[account(signer)]
    pub authority: Box<Account<'info, ControlAuthority>>,

    /// The payer of rent for new accounts
    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_pool_handler(ctx: Context<CreatePool>) -> Result<()> {
    let pool = &mut ctx.accounts.margin_pool;

    pool.address = pool.key();
    pool.pool_bump[0] = *ctx.bumps.get("margin_pool").unwrap();
    pool.token_mint = ctx.accounts.token_mint.key();
    pool.vault = ctx.accounts.vault.key();
    pool.deposit_note_mint = ctx.accounts.deposit_note_mint.key();
    pool.loan_note_mint = ctx.accounts.loan_note_mint.key();

    let clock = Clock::get()?;
    pool.accrued_until = clock.unix_timestamp;

    Ok(())
}
