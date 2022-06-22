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

use crate::{events, ErrorCode, MarginAccount};

#[derive(Accounts)]
pub struct CloseAccount<'info> {
    /// The owner of the account being closed
    pub owner: Signer<'info>,

    /// The account to get any returned rent
    /// CHECK:
    #[account(mut)]
    pub receiver: AccountInfo<'info>,

    /// The account being closed
    #[account(mut,
              close = receiver,
              has_one = owner)]
    pub margin_account: AccountLoader<'info, MarginAccount>,
}

pub fn close_account_handler(ctx: Context<CloseAccount>) -> Result<()> {
    let account = ctx.accounts.margin_account.load()?;

    if account.positions().count() > 0 {
        return Err(ErrorCode::AccountNotEmpty.into());
    }

    emit!(events::AccountClosed {
        margin_account: ctx.accounts.margin_account.key(),
    });

    Ok(())
}
