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
use crate::{events, ErrorCode, MarginAccount};

#[derive(Accounts)]
pub struct AdapterInvoke<'info> {
    /// The authority that owns the margin account
    pub owner: Signer<'info>,

    /// The margin account to proxy an action for
    #[account(mut, has_one = owner)]
    pub margin_account: AccountLoader<'info, MarginAccount>,
    //
    // see invoke_many doc for remaining_accounts structure
}

pub fn adapter_invoke_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, AdapterInvoke<'info>>,
    data: Vec<(u8, Vec<u8>)>,
) -> Result<()> {
    if ctx.accounts.margin_account.load()?.liquidation != Pubkey::default() {
        msg!("account is being liquidated");
        return Err(ErrorCode::Liquidating.into());
    }

    emit!(events::AdapterInvokeBegin {
        margin_account: ctx.accounts.margin_account.key(),
        adapter_program: Pubkey::default(), //todo
    });

    let events = adapter::invoke_many(
        &ctx.accounts.margin_account,
        ctx.remaining_accounts,
        data,
        true,
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
