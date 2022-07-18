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
use anchor_spl::token::{self, Burn, Token, TokenAccount, Transfer};

use crate::{
    events,
    state::{PartialAmount, PoolAction},
    ChangeKind, ErrorCode, MarginPool,
};

#[derive(Accounts)]
pub struct Repay<'info> {
    /// The pool with the outstanding loan
    #[account(
        mut,
        has_one = loan_note_mint,
        has_one = vault
    )]
    pub margin_pool: Box<Account<'info, MarginPool>>,

    /// The mint for the notes representing loans from the pool
    /// CHECK:
    #[account(mut)]
    pub loan_note_mint: AccountInfo<'info>,

    /// The vault responsible for storing the pool's tokens
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    /// The account with the loan notes
    #[account(mut)]
    pub loan_account: Account<'info, TokenAccount>,

    /// The token account repaying the debt
    #[account(mut)]
    pub repayment_token_account: Account<'info, TokenAccount>,

    /// Signing authority for the repaying token account
    pub repayment_account_authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> Repay<'info> {
    fn burn_loan_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                mint: self.loan_note_mint.to_account_info(),
                from: self.loan_account.to_account_info(),
                authority: self.margin_pool.to_account_info(),
            },
        )
    }

    fn transfer_repayment_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.repayment_token_account.to_account_info(),
                to: self.vault.to_account_info(),
                authority: self.repayment_account_authority.to_account_info(),
            },
        )
    }
}

pub fn repay_handler(ctx: Context<Repay>, change_kind: ChangeKind, amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.margin_pool;
    let clock = Clock::get()?;

    // Make sure interest accrual is up-to-date
    if !pool.accrue_interest(clock.unix_timestamp) {
        msg!("interest accrual is too far behind");
        return Err(ErrorCode::InterestAccrualBehind.into());
    }

    // Amount the user desires to repay
    let repay_amount = pool.calculate_full_amount(
        PartialAmount::tokens_to_loan_notes(amount),
        ctx.accounts.loan_account.amount,
        change_kind,
        PoolAction::Repay,
    )?;

    // Then record a repay using the withdrawn tokens
    msg!(
        "Repaying [{} tokens, {} notes] into loan pool",
        repay_amount.tokens,
        repay_amount.notes
    );
    pool.repay(&repay_amount)?;

    // Finish by transfering the requisite tokens and burning the loan notes
    let pool = &ctx.accounts.margin_pool;
    let signer = [&pool.signer_seeds()?[..]];

    token::transfer(
        ctx.accounts.transfer_repayment_context(),
        repay_amount.tokens,
    )?;
    token::burn(
        ctx.accounts.burn_loan_context().with_signer(&signer),
        repay_amount.notes,
    )?;

    emit!(events::Repay {
        margin_pool: pool.key(),
        loan_account: ctx.accounts.loan_account.key(),
        repayment_token_account: ctx.accounts.repayment_token_account.key(),
        repaid_tokens: repay_amount.tokens,
        repaid_loan_notes: repay_amount.notes,
        summary: (&pool.clone().into_inner()).into(),
    });
    Ok(())
}
