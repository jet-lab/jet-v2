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
use anchor_spl::token::{self, Burn, Token, TokenAccount};

use jet_margin::MarginAccount;
use jet_program_common::debug_msg;

use crate::{events, state::*, ChangeKind, TokenChange};
use crate::{Amount, ErrorCode};

#[derive(Accounts)]
pub struct MarginRepay<'info> {
    /// The margin account being executed on
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The pool with the outstanding loan
    #[account(mut,
              has_one = deposit_note_mint,
              has_one = loan_note_mint)]
    pub margin_pool: Account<'info, MarginPool>,

    /// The mint for the notes representing loans from the pool
    /// CHECK:
    #[account(mut)]
    pub loan_note_mint: AccountInfo<'info>,

    /// The mint for the notes representing deposit into the pool
    /// CHECK:
    #[account(mut)]
    pub deposit_note_mint: AccountInfo<'info>,

    /// The account with the loan notes
    #[account(mut)]
    pub loan_account: Account<'info, TokenAccount>,

    /// The account with the deposit to pay off the loan with
    #[account(mut)]
    pub deposit_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> MarginRepay<'info> {
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

    fn burn_deposit_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                from: self.deposit_account.to_account_info(),
                mint: self.deposit_note_mint.to_account_info(),
                authority: self.margin_account.to_account_info(),
            },
        )
    }
}

pub fn margin_repay_handler(
    ctx: Context<MarginRepay>,
    change_kind: ChangeKind,
    amount: u64,
) -> Result<()> {
    let change = TokenChange {
        kind: change_kind,
        tokens: amount,
    };
    debug_msg!(
        "Repaying {:?} towards loan of {} notes: {}",
        change,
        ctx.accounts.loan_account.amount,
        ctx.accounts.loan_account.key()
    );
    let pool = &mut ctx.accounts.margin_pool;
    let clock = Clock::get()?;

    // Make sure interest accrual is up-to-date
    if !pool.accrue_interest(clock.unix_timestamp) {
        msg!("interest accrual is too far behind");
        return Err(ErrorCode::InterestAccrualBehind.into());
    }

    // Amount the user desires to repay, and the amount of deposit notes equivalent to that repayment.
    let repay_amount =
        pool.calculate_full_amount(ctx.accounts.loan_account.amount, change, PoolAction::Repay)?;
    let withdraw_amount =
        pool.convert_amount(Amount::tokens(repay_amount.tokens), PoolAction::Withdraw)?;

    // Then record a repay using the withdrawn tokens
    pool.margin_repay(&repay_amount, &withdraw_amount)?;

    // Finish by burning the loan and deposit notes
    let pool = &ctx.accounts.margin_pool;
    let signer = [&pool.signer_seeds()?[..]];

    token::burn(
        ctx.accounts.burn_loan_context().with_signer(&signer),
        repay_amount.notes,
    )?;
    token::burn(ctx.accounts.burn_deposit_context(), withdraw_amount.notes)?;

    emit!(events::MarginRepay {
        margin_pool: pool.key(),
        user: ctx.accounts.margin_account.key(),
        loan_account: ctx.accounts.loan_account.key(),
        deposit_account: ctx.accounts.deposit_account.key(),
        repaid_tokens: repay_amount.tokens,
        repaid_loan_notes: repay_amount.notes,
        repaid_deposit_notes: withdraw_amount.notes,
        summary: pool.deref().into(),
    });

    Ok(())
}
