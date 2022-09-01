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
    events::AirspaceDirectivesChanged,
    state::{Airspace, Directives, GovernorId},
};

#[derive(Accounts)]
pub struct AirspaceSetDirectives<'info> {
    /// The current airspace governor
    governor: Signer<'info>,

    /// The identity account for the governor
    #[account(has_one = governor)]
    governor_id: Account<'info, GovernorId>,

    /// The airspace to have its directives altered
    #[account(mut)]
    airspace: Account<'info, Airspace>,
}

pub fn airspace_set_directives_handler(
    ctx: Context<AirspaceSetDirectives>,
    new_directives: Directives,
) -> Result<()> {
    let airspace = &mut ctx.accounts.airspace;

    airspace.directives = new_directives;

    emit!(AirspaceDirectivesChanged {
        new_directives,
        airspace: airspace.key(),
    });

    Ok(())
}
