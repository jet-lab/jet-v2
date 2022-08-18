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

//!## adapter\_invoke.rs
//!
//!This instruction does the following:
//!
//!1.  If a read account has the `liquidation` parameter set to a pubkey:
//!    
//!    1.  This means that that margin account is already under liquidation by the liquidator at that pubkey.
//!        
//!    2.  Return `ErrorCode::Liquidating`.
//!        
//!2.  Emit the `AdapterInvokeBegin` event for data logging (see table below).
//!    
//!3.  Check if any positions that have changed via adapters.
//!    
//!    1.  For each changed position, emit each existing adapter position as an `event` (see table below).
//!        
//!4.  Emit the `AdapterInvokeEnd` event for data logging (see table below).
//!    
//!5.  Verify that margin accounts positions via adapter are healthy.
//!    
//!6.  Return `Ok(())`.
//!    
//!
//!**Parameters of adapter\_invoke.rs:**
//!
//!|     |     |
//!| --- | --- |
//!| **Name** | **Description** |
//!| `owner` | The authority that owns the margin account. |
//!| `margin_account` | The margin account to proxy an action for. |
//!| `adapter_program` | The program to be invoked. |
//!| `adapter_metadata` | The metadata about the proxy program. |
//!
//!**Events emitted by adapter\_invoke.rs:**
//!
//!|     |     |
//!| --- | --- |
//!| **Event Name** | **Description** |
//!| `AdapterInvokeBegin` | Marks the start of the adapter invocation (includes the margin account pubkey and the adapter program pubkey). |
//!| `event` _(Note that each single event represents a different adapter position)_ | Each adapter position is emitted as an event (includes the margin account, the adapter program, the accounts, and a value of `true` for the field `signed`. |
//!| `AdapterInvokeEnd` | Marks the ending of the adapter invocation (includes no data except for the event itself being emitted). |
//!

use anchor_lang::prelude::*;

use jet_metadata::MarginAdapterMetadata;

use crate::adapter::{self, InvokeAdapter};
use crate::{events, ErrorCode, MarginAccount};

#[derive(Accounts)]
pub struct AdapterInvoke<'info> {
    /// The authority that owns the margin account
    pub owner: Signer<'info>,

    /// The margin account to proxy an action for
    #[account(mut, has_one = owner)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The program to be invoked
    /// CHECK:
    pub adapter_program: AccountInfo<'info>,

    /// The metadata about the proxy program
    #[account(has_one = adapter_program)]
    pub adapter_metadata: Account<'info, MarginAdapterMetadata>,
}

pub fn adapter_invoke_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, AdapterInvoke<'info>>,
    data: Vec<u8>,
) -> Result<()> {
    if ctx.accounts.margin_account.load()?.liquidation != Pubkey::default() {
        msg!("account is being liquidated");
        return Err(ErrorCode::Liquidating.into());
    }

    emit!(events::AdapterInvokeBegin {
        margin_account: ctx.accounts.margin_account.key(),
        adapter_program: ctx.accounts.adapter_program.key(),
    });

    let events = adapter::invoke(
        &InvokeAdapter {
            margin_account: &ctx.accounts.margin_account,
            adapter_program: &ctx.accounts.adapter_program,
            accounts: ctx.remaining_accounts,
            signed: true,
        },
        data,
    )?;

    for event in events {
        event.emit();
    }

    emit!(events::AdapterInvokeEnd {});

    ctx.accounts
        .margin_account
        .load()?
        .verify_healthy_positions()?;

    Ok(())
}
