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
use anchor_spl::token::{self, MintTo, Token, Transfer};

use crate::{events, state::*, ChangeKind, ErrorCode};

#[derive(Accounts)]
pub struct Deposit<'info> {
    /// The pool to deposit into
    #[account(mut,
              has_one = vault,
              has_one = deposit_note_mint)]
    pub margin_pool: Account<'info, MarginPool>,

    /// The vault for the pool, where tokens are held
    /// CHECK:
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,

    /// The mint for the deposit notes
    /// CHECK:
    #[account(mut)]
    pub deposit_note_mint: UncheckedAccount<'info>,

    /// The address with authority to deposit the tokens
    pub depositor: Signer<'info>,

    /// The source of the tokens to be deposited
    /// CHECK:
    #[account(mut)]
    pub source: UncheckedAccount<'info>,

    /// The destination of the deposit notes
    /// CHECK:
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> Deposit<'info> {
    fn transfer_source_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                to: self.vault.to_account_info(),
                from: self.source.to_account_info(),
                authority: self.depositor.to_account_info(),
            },
        )
    }

    fn mint_note_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                to: self.destination.to_account_info(),
                mint: self.deposit_note_mint.to_account_info(),
                authority: self.margin_pool.to_account_info(),
            },
        )
    }
}

pub fn deposit_handler(ctx: Context<Deposit>, change_kind: ChangeKind, amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.margin_pool;
    let clock = Clock::get()?;

    // Make sure interest accrual is up-to-date
    if !pool.accrue_interest(clock.unix_timestamp) {
        msg!("interest accrual is too far behind");
        return Err(ErrorCode::InterestAccrualBehind.into());
    }

    let deposit_amount = pool.calculate_dispersement(
        amount,
        token::accessor::amount(&ctx.accounts.destination.to_account_info())?,
        change_kind,
        PoolAction::Deposit,
    )?;
    pool.deposit(&deposit_amount);

    let pool = &ctx.accounts.margin_pool;
    let signer = [&pool.signer_seeds()?[..]];

    token::transfer(
        ctx.accounts.transfer_source_context().with_signer(&signer),
        deposit_amount.tokens,
    )?;
    token::mint_to(
        ctx.accounts.mint_note_context().with_signer(&signer),
        deposit_amount.notes,
    )?;

    emit!(events::Deposit {
        margin_pool: ctx.accounts.margin_pool.key(),
        user: ctx.accounts.depositor.key(),
        source: ctx.accounts.source.key(),
        destination: ctx.accounts.destination.key(),
        deposit_tokens: deposit_amount.tokens,
        deposit_notes: deposit_amount.notes,
        summary: pool.deref().into(),
    });

    Ok(())
}
