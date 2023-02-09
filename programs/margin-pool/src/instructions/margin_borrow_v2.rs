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

use std::ops::Deref;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Token, TokenAccount, Transfer};

use jet_margin::MarginAccount;

use crate::{events, state::*, ChangeKind, ErrorCode, TokenChange};

#[derive(Accounts)]
pub struct MarginBorrowV2<'info> {
    /// The margin account being executed on
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The pool to borrow from
    #[account(mut,
              has_one = vault,
              has_one = loan_note_mint)]
    pub margin_pool: Account<'info, MarginPool>,

    /// The vault responsible for storing the pool's tokens
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    /// The mint for the notes representing loans from the pool
    /// CHECK:
    #[account(mut)]
    pub loan_note_mint: AccountInfo<'info>,

    /// The account to receive the loan notes
    #[account(mut,
        constraint = loan_account.owner == margin_pool.key(),
        seeds = [margin_account.key().as_ref(),
                 loan_note_mint.key().as_ref()],
        bump,
    )]
    pub loan_account: Account<'info, TokenAccount>,

    /// The account to receive the borrowed tokens
    #[account(mut)]
    pub destination: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> MarginBorrowV2<'info> {
    fn mint_loan_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.loan_note_mint.to_account_info(),
                to: self.loan_account.to_account_info(),
                authority: self.margin_pool.to_account_info(),
            },
        )
    }

    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                to: self.destination.to_account_info(),
                from: self.vault.to_account_info(),
                authority: self.margin_pool.to_account_info(),
            },
        )
    }
}

#[inline(never)]
pub fn margin_borrow_v2_handler(ctx: Context<MarginBorrowV2>, amount: u64) -> Result<()> {
    let change = TokenChange {
        kind: ChangeKind::ShiftBy,
        tokens: amount,
    };
    let pool = &mut ctx.accounts.margin_pool;
    let clock = Clock::get()?;

    // Make sure interest accrual is up-to-date
    if !pool.accrue_interest(clock.unix_timestamp) {
        msg!("interest accrual is too far behind");
        return Err(ErrorCode::InterestAccrualBehind.into());
    }

    // Record a borrow of the tokens requested
    let borrow_amount =
        pool.calculate_full_amount(ctx.accounts.loan_account.amount, change, PoolAction::Borrow)?;
    pool.borrow(&borrow_amount)?;

    // Finish by minting the loan notes
    let pool = &ctx.accounts.margin_pool;
    let signer = [&pool.signer_seeds()?[..]];

    token::mint_to(
        ctx.accounts.mint_loan_context().with_signer(&signer),
        borrow_amount.notes,
    )?;
    token::transfer(
        ctx.accounts.transfer_context().with_signer(&signer),
        borrow_amount.tokens,
    )?;

    emit!(events::MarginBorrow {
        margin_pool: ctx.accounts.margin_pool.key(),
        user: ctx.accounts.margin_account.key(),
        loan_account: ctx.accounts.loan_account.key(),
        deposit_account: ctx.accounts.destination.key(),
        tokens: borrow_amount.tokens,
        loan_notes: borrow_amount.notes,
        deposit_notes: 0,
        summary: pool.deref().into(),
    });

    Ok(())
}
