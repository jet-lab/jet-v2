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

use crate::{
    seeds::DEFAULT_DIRECTIVES,
    state::{DefaultDirectives, Directives, GovernorId},
};

#[derive(Accounts)]
pub struct SetDefaultDirectives<'info> {
    /// Payer for rent
    #[account(mut)]
    payer: Signer<'info>,

    /// The current airspace governor
    governor: Signer<'info>,

    /// The identity account for the governor
    #[account(has_one = governor)]
    governor_id: Account<'info, GovernorId>,

    /// The default directives account
    #[account(init_if_needed,
              seeds = [DEFAULT_DIRECTIVES],
              bump,
              payer = payer,
              space = DefaultDirectives::SIZE
    )]
    default_directives: Account<'info, DefaultDirectives>,

    system_program: Program<'info, System>,
}

pub fn set_default_directives_handler(
    ctx: Context<SetDefaultDirectives>,
    new_directives: Directives,
) -> Result<()> {
    let default = &mut ctx.accounts.default_directives;

    ***default = new_directives;

    Ok(())
}
