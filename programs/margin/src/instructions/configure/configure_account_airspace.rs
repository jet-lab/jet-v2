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

use jet_program_common::DEFAULT_AIRSPACE;

use crate::{ErrorCode, MarginAccount};

#[derive(Accounts)]
pub struct ConfigureAccountAirspace<'info> {
    /// The account to be configured
    #[account(mut)]
    pub margin_account: AccountLoader<'info, MarginAccount>,
}

pub fn configure_account_airspace_handler(ctx: Context<ConfigureAccountAirspace>) -> Result<()> {
    let margin_account = &mut ctx.accounts.margin_account.load_mut()?;

    if margin_account.airspace != Pubkey::default() {
        return Err(ErrorCode::AlreadyJoinedAirspace.into());
    }

    margin_account.airspace = DEFAULT_AIRSPACE;
    Ok(())
}
