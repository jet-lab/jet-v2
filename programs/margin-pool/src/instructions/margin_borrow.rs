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
use anchor_spl::token::{self, MintTo, Token, TokenAccount};

use jet_margin::{AdapterResult, MarginAccount, PositionChange};

use crate::{state::*, AmountKind};
use crate::{Amount, ErrorCode};

#[derive(Accounts)]
pub struct MarginBorrow<'info> {
    /// The margin account being executed on
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The pool to borrow from
    #[account(mut,
              has_one = loan_note_mint,
              has_one = deposit_note_mint)]
    pub margin_pool: Account<'info, MarginPool>,

    /// The mint for the notes representing loans from the pool
    /// CHECK:
    #[account(mut)]
    pub loan_note_mint: AccountInfo<'info>,

    /// The mint for the notes representing deposit into the pool
    /// CHECK:
    #[account(mut)]
    pub deposit_note_mint: AccountInfo<'info>,

    /// The account to receive the loan notes
    #[account(mut, constraint = loan_account.owner == margin_account.key())]
    pub loan_account: Account<'info, TokenAccount>,

    /// The account to receive the borrowed tokens (as deposit notes)
    #[account(mut, constraint = deposit_account.owner == margin_account.key())]
    pub deposit_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> MarginBorrow<'info> {
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

    fn mint_deposit_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                to: self.deposit_account.to_account_info(),
                mint: self.deposit_note_mint.to_account_info(),
                authority: self.margin_pool.to_account_info(),
            },
        )
    }
}

pub fn margin_borrow_handler(ctx: Context<MarginBorrow>, token_amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.margin_pool;
    let clock = Clock::get()?;

    // Make sure interest accrual is up-to-date
    if !pool.accrue_interest(clock.unix_timestamp) {
        msg!("interest accrual is too far behind");
        return Err(ErrorCode::InterestAccrualBehind.into());
    }

    // First record a borrow of the tokens requested
    let borrow_rounding = RoundingDirection::direction(PoolAction::Borrow, AmountKind::Tokens);
    let borrow_amount = pool.convert_loan_amount(Amount::tokens(token_amount), borrow_rounding)?;
    pool.borrow(&borrow_amount)?;

    // Then record a deposit of the same borrowed tokens
    let deposit_rounding = RoundingDirection::direction(PoolAction::Deposit, AmountKind::Tokens);
    let deposit_amount =
        pool.convert_deposit_amount(Amount::tokens(token_amount), deposit_rounding)?;
    pool.deposit(&deposit_amount);

    // Finish by minting the loan and deposit notes
    let pool = &ctx.accounts.margin_pool;
    let signer = [&pool.signer_seeds()?[..]];

    token::mint_to(
        ctx.accounts.mint_loan_context().with_signer(&signer),
        borrow_amount.notes,
    )?;
    token::mint_to(
        ctx.accounts.mint_deposit_context().with_signer(&signer),
        deposit_amount.notes,
    )?;

    jet_margin::write_adapter_result(&AdapterResult {
        position_changes: vec![(
            pool.loan_note_mint,
            vec![PositionChange::Expect(ctx.accounts.loan_account.key())],
        )],
    })?;

    Ok(())
}
