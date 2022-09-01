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
    events::AirspaceCreated,
    seeds::{AIRSPACE, DEFAULT_DIRECTIVES},
    state::{Airspace, DefaultDirectives},
};

#[derive(Accounts)]
#[instruction(seed: String)]
pub struct AirspaceCreate<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    /// The airspace account to be created
    #[account(init,
              seeds = [AIRSPACE, seed.as_ref()],
              bump,
              payer = payer,
              space = Airspace::SIZE
    )]
    airspace: Account<'info, Airspace>,

    /// The default configuration for resources
    #[account(seeds = [DEFAULT_DIRECTIVES], bump)]
    default_directives: Account<'info, DefaultDirectives>,

    system_program: Program<'info, System>,
}

pub fn airspace_create_handler(
    ctx: Context<AirspaceCreate>,
    _seed: String,
    is_restricted: bool,
    authority: Pubkey,
) -> Result<()> {
    let airspace = &mut ctx.accounts.airspace;

    airspace.authority = authority;
    airspace.directives = **ctx.accounts.default_directives;
    airspace.is_restricted = is_restricted;

    emit!(AirspaceCreated {
        airspace: airspace.key()
    });

    Ok(())
}
