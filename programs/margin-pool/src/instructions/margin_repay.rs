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
use jet_proto_math::Number;

use crate::ErrorCode;
use crate::{events, state::*, ChangeKind, TokenChange};

#[derive(Accounts)]
pub struct MarginRepay<'info> {
    /// The margin account being executed on
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The pool with the outstanding loan
    #[account(mut,
              has_one = deposit_note_mint,
              has_one = loan_note_mint,
              has_one = vault)]
    pub margin_pool: Account<'info, MarginPool>,

    pub vault: AccountInfo<'info>,

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
    let mut pool = ctx
        .accounts
        .margin_pool
        .join_mut()
        .with_vault(&ctx.accounts.vault)
        .with_loan_note_mint(&ctx.accounts.loan_note_mint)
        .with_deposit_note_mint(&ctx.accounts.deposit_note_mint);
    let clock = Clock::get()?;

    // Make sure interest accrual is up-to-date
    if !pool.accrue_interest(clock.unix_timestamp)? {
        msg!("interest accrual is too far behind");
        return Err(ErrorCode::InterestAccrualBehind.into());
    }

    // Amount the user desires to repay
    let repay_amount = pool.loan_amount()?.from_request(
        token::accessor::amount(&ctx.accounts.loan_account.to_account_info())?,
        change,
        PoolAction::Repay,
    )?;
    let repay_tokens = repay_amount.as_token_transfer(FromUser);
    let repay_notes = repay_amount.as_note_transfer(FromUser);

    // First record a withdraw of the deposit to use for repaying in tokens
    let withdraw_notes = pool
        .deposit_amount()?
        .from_tokens(Number::from(repay_tokens))
        .as_note_transfer(FromUser);

    msg!("Withdrawing [{} notes] from deposit pool", withdraw_notes);

    // Then record a repay using the withdrawn tokens
    msg!(
        "Repaying [{} tokens, {} notes] into loan pool",
        repay_tokens,
        repay_notes
    );
    pool.pool.repay(repay_tokens)?;

    // Finish by burning the loan and deposit notes
    let pool = &ctx.accounts.margin_pool;
    let signer = [&pool.signer_seeds()?[..]];

    token::burn(
        ctx.accounts.burn_loan_context().with_signer(&signer),
        repay_notes,
    )?;
    token::burn(
        ctx.accounts.burn_deposit_context().with_signer(&signer),
        withdraw_notes,
    )?;

    emit!(events::MarginRepay {
        margin_pool: pool.key(),
        user: ctx.accounts.margin_account.key(),
        loan_account: ctx.accounts.loan_account.key(),
        deposit_account: ctx.accounts.deposit_account.key(),
        repaid_tokens: repay_tokens,
        repaid_loan_notes: repay_notes,
        repaid_deposit_notes: withdraw_notes,
        summary: pool.deref().into(),
    });

    Ok(())
}
