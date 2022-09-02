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

use crate::adapter::{self, InvokeAdapter};
use crate::MarginAccount;

#[derive(Accounts)]
pub struct UserInvoke<'info> {
    /// The margin account to proxy an action for
    #[account(mut)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The program to be invoked
    pub program: AccountInfo<'info>,
}

pub fn user_invoke_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, UserInvoke<'info>>,
    data: Vec<u8>,
) -> Result<()> {
    let account = ctx.accounts.margin_account.load()?;

    account.tx_bound.verify_in_bound()?;

    // Do what the user is asking
    adapter::invoke(
        &InvokeAdapter {
            margin_account: &ctx.accounts.margin_account,
            adapter_program: &ctx.accounts.program,
            accounts: ctx.remaining_accounts,
            signed: true,
        },
        data,
    )?
    .into_iter()
    .for_each(|event| event.emit());

    Ok(())
}
