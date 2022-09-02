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
use anchor_lang::solana_program::sysvar::instructions::ID as SYSVAR_INSTRUCTIONS_ID;

use crate::MarginAccount;

#[derive(Accounts)]
pub struct UserEndTransaction<'info> {
    /// The margin account to be validated
    #[account(mut)]
    margin_account: AccountLoader<'info, MarginAccount>,

    /// The instructions sysvar
    #[account(address = SYSVAR_INSTRUCTIONS_ID)]
    pub instructions: AccountInfo<'info>,
}

pub fn user_end_transaction_handler(ctx: Context<UserEndTransaction>) -> Result<()> {
    let mut account = ctx.accounts.margin_account.load_mut()?;

    // FIXME: do liquidator check instead if its this is a liquidation tx
    account.verify_healthy_positions()?;

    // Verify that this is the end of the user transaction
    account.tx_bound.end(&ctx.accounts.instructions)?;

    Ok(())
}
