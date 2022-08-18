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

use crate::adapter::{self, InvokeAdapter};
use crate::{events, MarginAccount};

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
/// ## accounting\_invoke.rs
/// 
/// This instruction does the following:
/// 
/// 1.  Emit `AccountingInvokeBegin` events for data logging (see table below).
///     
/// 2.  Check if any positions that have changed via adapters.
///     
///     a.  For each changed position, emit each existing adapter position as an `event` (see table below).
///         
/// 3.  Emit `AccountingInvokeEnd` event for data logging (see table below).
///     
/// 4.  Return `Ok(())`.
///     
/// 
/// !**Parameters of accounting\_invoke.rs:**
/// 
/// |     |     |
/// | --- | --- |
/// | **Name** | **Description** |
/// | `margin_account` | The margin account to proxy an action for. |
/// | `adapter_program` | The program to be invoked. |
/// | `adapter_metadata` | The metadata about the proxy program. |
/// 
/// **Events emitted by accounting\_invoke.rs:**
/// 
/// |     |     |
/// | --- | --- |
/// | **Name** | **Description** |
/// | `AccountingInvokeBegin` | Signify that the accounting invocation process has begun. |
/// | `event` | Each adapter position is emitted as an event (includes the margin account, the adapter program, the remaining accounts, and a value of `false` for the field `signed`. |
/// | `AccountingInvokeEnd` | The margin account to proxy an action for. |

pub fn accounting_invoke_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, AccountingInvoke<'info>>,
    data: Vec<u8>,
) -> Result<()> {
    emit!(events::AccountingInvokeBegin {
        margin_account: ctx.accounts.margin_account.key(),
        adapter_program: ctx.accounts.adapter_program.key(),
    });

    let events = adapter::invoke(
        &InvokeAdapter {
            margin_account: &ctx.accounts.margin_account,
            adapter_program: &ctx.accounts.adapter_program,
            accounts: ctx.remaining_accounts,
            signed: false,
        },
        data,
    )?;

    for event in events {
        event.emit();
    }

    emit!(events::AccountingInvokeEnd {});

    Ok(())
}
