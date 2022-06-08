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
use crate::{ErrorCode, MarginAccount};

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
    account_metas: Vec<CompactAccountMeta>,
    data: Vec<u8>,
) -> Result<()> {
    if ctx.accounts.margin_account.load()?.liquidation != Pubkey::default() {
        msg!("account is being liquidated");
        return Err(ErrorCode::Liquidating.into());
    }

    adapter::invoke_signed(
        &InvokeAdapter {
            margin_account: &ctx.accounts.margin_account,
            adapter_program: &ctx.accounts.adapter_program,
            remaining_accounts: ctx.remaining_accounts,
        },
        account_metas,
        data,
    )?;

    ctx.accounts
        .margin_account
        .load()?
        .verify_healthy_positions()?;

    Ok(())
}
