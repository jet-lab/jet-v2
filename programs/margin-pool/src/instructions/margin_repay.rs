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
use anchor_spl::token::{self, Burn, Token, TokenAccount};

use jet_margin::MarginAccount;

use crate::state::*;
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
                to: self.loan_account.to_account_info(),
                authority: self.margin_account.to_account_info(),
            },
        )
    }

    fn burn_deposit_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                to: self.deposit_account.to_account_info(),
                mint: self.deposit_note_mint.to_account_info(),
                authority: self.margin_account.to_account_info(),
            },
        )
    }
}

pub fn margin_repay_handler(ctx: Context<MarginRepay>, max_amount: Amount) -> Result<()> {
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
    let withdraw_note_rounding =
        RoundingDirection::direction(PoolAction::Withdraw, crate::AmountKind::Notes);
    let withdraw_token_rounding =
        RoundingDirection::direction(PoolAction::Withdraw, crate::AmountKind::Tokens);

    // Determine the maximum notes repayable and withdrawable from each account
    let loan_notes = ctx.accounts.loan_account.amount;
    let deposit_notes = ctx.accounts.deposit_account.amount;

    // Amount the user desires to repay
    let desired_repay_amount = pool.convert_loan_amount(max_amount, repay_rounding)?;
    // Maximum amount deposited
    let max_withdraw_amount =
        pool.convert_deposit_amount(Amount::notes(deposit_notes), withdraw_note_rounding)?;
    // Maximum amount owed
    let max_repay_amount =
        pool.convert_loan_amount(Amount::notes(loan_notes), repay_note_rounding)?;

    // Determine the maximum tokens to withdraw and repay, as the lower of the 3 above
    let max_repay_tokens = desired_repay_amount
        .tokens
        .min(max_withdraw_amount.tokens)
        .min(max_repay_amount.tokens);

    // First record a withdraw of the deposit to use for repaying in tokens
    let withdraw_amount =
        pool.convert_deposit_amount(Amount::tokens(max_repay_tokens), withdraw_token_rounding)?;
    pool.withdraw(&withdraw_amount)?;

    // Then record a repay using the withdrawn tokens
    let repay_amount =
        pool.convert_loan_amount(Amount::tokens(max_repay_tokens), repay_token_rounding)?;
    pool.repay(&repay_amount)?;

    // Finish by burning the loan and deposit notes
    let pool = &ctx.accounts.margin_pool;
    let signer = [&pool.signer_seeds()?[..]];

    token::burn(
        ctx.accounts.burn_loan_context().with_signer(&signer),
        repay_amount.notes,
    )?;
    token::burn(
        ctx.accounts.burn_deposit_context().with_signer(&signer),
        withdraw_amount.notes,
    )?;

    Ok(())
}
