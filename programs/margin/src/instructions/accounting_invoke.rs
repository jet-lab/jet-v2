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
use crate::{events, AdapterConfig, ErrorCode, MarginAccount};

#[derive(Accounts)]
pub struct AccountingInvoke<'info> {
    /// The margin account to proxy an action for
    #[account(mut)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The program to be invoked
    /// CHECK:
    pub adapter_program: AccountInfo<'info>,

    /// The metadata about the proxy program
    #[account(has_one = adapter_program,
              constraint = adapter_config.airspace == margin_account.load()?.airspace @ ErrorCode::WrongAirspace
    )]
    pub adapter_config: Account<'info, AdapterConfig>,
}

pub fn accounting_invoke_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, AccountingInvoke<'info>>,
    data: Vec<u8>,
) -> Result<()> {
    emit!(events::AccountingInvokeBegin {
        margin_account: ctx.accounts.margin_account.key(),
        adapter_program: ctx.accounts.adapter_program.key(),
    });

    adapter::invoke(
        &InvokeAdapter {
            margin_account: &ctx.accounts.margin_account,
            adapter_program: &ctx.accounts.adapter_program,
            accounts: ctx.remaining_accounts,
            signed: false,
        },
        data,
    )?;

    emit!(events::AccountingInvokeEnd {});

    Ok(())
}
