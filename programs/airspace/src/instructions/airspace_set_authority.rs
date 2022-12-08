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

use crate::{events::AirspaceAuthoritySet, state::Airspace};

#[derive(Accounts)]
pub struct AirspaceSetAuthority<'info> {
    /// The current airspace authority
    authority: Signer<'info>,

    /// The airspace to have its authority changed
    #[account(mut, has_one = authority)]
    airspace: Account<'info, Airspace>,
}

pub fn airspace_set_authority_handler(
    ctx: Context<AirspaceSetAuthority>,
    new_authority: Pubkey,
) -> Result<()> {
    let airspace = &mut ctx.accounts.airspace;

    airspace.authority = new_authority;

    emit!(AirspaceAuthoritySet {
        airspace: airspace.key(),
        authority: new_authority
    });

    Ok(())
}
