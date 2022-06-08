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

use jet_metadata::MarginAdapterMetadata;

use crate::adapter::{self, CompactAccountMeta, InvokeAdapter};
use crate::MarginAccount;

#[derive(Accounts)]
pub struct AccountingInvoke<'info> {
    /// The margin account to proxy an action for
    #[account(mut)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The program to be invoked
    /// CHECK:
    pub adapter_program: AccountInfo<'info>,

    /// The metadata about the proxy program
    #[account(has_one = adapter_program)]
    pub adapter_metadata: Account<'info, MarginAdapterMetadata>,
}

pub fn accounting_invoke_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, AccountingInvoke<'info>>,
    account_metas: Vec<CompactAccountMeta>,
    data: Vec<u8>,
) -> Result<()> {
    adapter::invoke(
        &InvokeAdapter {
            margin_account: &ctx.accounts.margin_account,
            adapter_program: &ctx.accounts.adapter_program,
            remaining_accounts: ctx.remaining_accounts,
        },
        account_metas,
        data,
    )
}
