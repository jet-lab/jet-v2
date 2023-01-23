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
use anchor_spl::token::TokenAccount;

use crate::{
    events,
    syscall::{sys, Sys},
    MarginAccount,
};

#[derive(Accounts)]
pub struct UpdatePositionBalance<'info> {
    /// The account to update
    #[account(mut)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The token account to update the balance for
    pub token_account: Account<'info, TokenAccount>,
}

pub fn update_position_balance_handler(ctx: Context<UpdatePositionBalance>) -> Result<()> {
    let mut margin_account = ctx.accounts.margin_account.load_mut()?;
    let token_account = &ctx.accounts.token_account;

    let position = margin_account.set_position_balance(
        &token_account.mint,
        &token_account.key(),
        token_account.amount,
        sys().unix_timestamp(),
    )?;

    emit!(events::PositionBalanceUpdated { position });

    Ok(())
}
