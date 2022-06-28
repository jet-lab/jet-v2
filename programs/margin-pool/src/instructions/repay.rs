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
use anchor_spl::token::{self, Burn, Token, TokenAccount, Transfer};
use jet_margin::MarginAccount;

use crate::{
    events,
    state::{PoolAction, RoundingDirection},
    Amount, ErrorCode, MarginPool,
};

#[derive(Accounts)]
pub struct Repay<'info> {
    /// The margin account being executed on
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The pool with the outstanding loan
    #[account(
        mut,
        has_one = loan_note_mint,
        constraint = margin_pool.vault == pool_vault.key()
    )]
    pub margin_pool: Account<'info, MarginPool>,

    /// The mint for the notes representing loans from the pool
    /// CHECK:
    #[account(mut)]
    pub loan_note_mint: AccountInfo<'info>,

    /// The vault responsible for storing the pool's tokens
    #[account(mut)]
    pub pool_vault: Account<'info, TokenAccount>,

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
                to: self.loan_account.to_account_info(),
                authority: self.margin_account.to_account_info(),
            },
        )
    }

    fn transfer_repayment_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.repayment_token_account.to_account_info(),
                to: self.pool_vault.to_account_info(),
                authority: self.repayment_account_authority.to_account_info(),
            },
        )
    }
}

pub fn repay_handler(ctx: Context<Repay>, max_amount: Amount) -> Result<()> {
    let pool = &mut ctx.accounts.margin_pool;
    let clock = Clock::get()?;

    // Make sure interest accrual is up-to-date
    if !pool.accrue_interest(clock.unix_timestamp) {
        msg!("interest accrual is too far behind");
        return Err(ErrorCode::InterestAccrualBehind.into());
    }

    // The rounding to repay the maximum amount specified
    let repay_rounding = RoundingDirection::direction(PoolAction::Repay, max_amount.kind);

    // const rounding directions
    let repay_note_rounding =
        RoundingDirection::direction(PoolAction::Repay, crate::AmountKind::Notes);
    let repay_token_rounding =
        RoundingDirection::direction(PoolAction::Repay, crate::AmountKind::Tokens);

    let loan_notes = ctx.accounts.loan_account.amount;

    // Amount the user desires to repay
    let desired_repay_amount = pool.convert_loan_amount(max_amount, repay_rounding)?;
    // Maximum amount for repayment
    let max_transfer_amount = ctx.accounts.repayment_token_account.amount;
    // Maximum amount owed
    let max_repay_amount =
        pool.convert_loan_amount(Amount::notes(loan_notes), repay_note_rounding)?;

    // Determine the maximum tokens to transfer and repay, as the lower of the 3 above
    let max_repay_tokens = desired_repay_amount
        .tokens
        .min(max_transfer_amount)
        .min(max_repay_amount.tokens);

    // Then record a repay using the withdrawn tokens
    let repay_amount =
        pool.convert_loan_amount(Amount::tokens(max_repay_tokens), repay_token_rounding)?;
    msg!(
        "Repaying [{} tokens, {} notes] into loan pool",
        repay_amount.tokens,
        repay_amount.notes
    );
    pool.repay(&repay_amount)?;

    // Finish by transfering the requisite tokens and burning the loan notes
    let pool = &ctx.accounts.margin_pool;
    let signer = [&pool.signer_seeds()?[..]];

    token::transfer(ctx.accounts.transfer_repayment_context(), max_repay_tokens)?;
    token::burn(
        ctx.accounts.burn_loan_context().with_signer(&signer),
        repay_amount.notes,
    )?;

    emit!(events::Repay {
        margin_pool: pool.key(),
        user: ctx.accounts.margin_account.key(),
        loan_account: ctx.accounts.loan_account.key(),
        repayment_token_account: ctx.accounts.repayment_token_account.key(),
        max_repay_tokens: desired_repay_amount.tokens,
        max_repay_notes: desired_repay_amount.notes,
        repaid_tokens: max_repay_tokens,
        repaid_loan_notes: repay_amount.notes,
        summary: pool.deref().into(),
    });

    Ok(())
}
