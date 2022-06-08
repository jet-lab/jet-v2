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

use std::collections::BTreeMap;

use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use jet_margin::MarginAccount;

use crate::state::*;
use crate::Amount;

#[derive(Accounts)]
pub struct MarginWithdraw<'info> {
    /// The margin account being executed on
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The pool to withdraw from
    #[account(mut,
              has_one = vault,
              has_one = deposit_note_mint)]
    pub margin_pool: Account<'info, MarginPool>,

    /// The vault for the pool, where tokens are held
    /// CHECK:
    #[account(mut)]
    pub vault: AccountInfo<'info>,

    /// The mint for the deposit notes
    /// CHECK:
    #[account(mut)]
    pub deposit_note_mint: UncheckedAccount<'info>,

    /// The source of the deposit notes to be redeemed
    /// CHECK:
    #[account(mut)]
    pub source: UncheckedAccount<'info>,

    /// The destination of the tokens withdrawn
    /// CHECK:
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn margin_withdraw_handler(ctx: Context<MarginWithdraw>, amount: Amount) -> Result<()> {
    // PERF: ?
    // just forward to normal withdraw handling
    super::withdraw_handler(
        Context::new(
            ctx.program_id,
            &mut super::Withdraw {
                margin_pool: ctx.accounts.margin_pool.clone(),
                vault: ctx.accounts.vault.clone(),
                deposit_note_mint: ctx.accounts.deposit_note_mint.clone(),
                depositor: Signer::try_from(&ctx.accounts.margin_account.to_account_info())?,
                source: ctx.accounts.source.clone(),
                destination: ctx.accounts.destination.clone(),
                token_program: ctx.accounts.token_program.clone(),
            },
            &[],
            BTreeMap::new(),
        ),
        amount,
    )?;

    Ok(())
}
