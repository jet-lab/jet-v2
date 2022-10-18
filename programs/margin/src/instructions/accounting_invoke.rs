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

use crate::adapter;
use crate::{events, MarginAccount};

#[derive(Accounts)]
pub struct AccountingInvoke<'info> {
    /// The margin account to proxy an action for
    #[account(mut)]
    pub margin_account: AccountLoader<'info, MarginAccount>,
    //
    // see invoke_many doc for remaining_accounts structure
}

pub fn accounting_invoke_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, AccountingInvoke<'info>>,
    data: Vec<(u8, Vec<u8>)>,
) -> Result<()> {
    emit!(events::AccountingInvokeBegin {
        margin_account: ctx.accounts.margin_account.key(),
        adapter_program: Pubkey::default(), //todo
    });

    let events = adapter::invoke_many(
        &ctx.accounts.margin_account,
        ctx.remaining_accounts,
        data,
        false,
    )?;

    for event in events {
        event.emit();
    }

    emit!(events::AccountingInvokeEnd {});

    Ok(())
}
