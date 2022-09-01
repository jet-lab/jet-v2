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
    events::AirspacePermitCreated,
    seeds::{AIRSPACE_PERMIT, AIRSPACE_PERMIT_ISSUER},
    state::{Airspace, AirspacePermit, AirspacePermitIssuerId},
};

#[derive(Accounts)]
#[instruction(owner: Pubkey)]
pub struct AirspacePermitCreate<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    /// The authority allowed to create a permit in the airspace
    ///
    /// If the airspace is restricted, then this must be either the airspace authority or
    /// an authorized regulator.
    authority: Signer<'info>,

    /// The airspace the new permit is for
    airspace: Account<'info, Airspace>,

    /// The airspace account to be created
    #[account(init,
              seeds = [
                AIRSPACE_PERMIT,
                airspace.key().as_ref(),
                owner.as_ref()
              ],
              bump,
              payer = payer,
              space = AirspacePermit::SIZE
    )]
    permit: Account<'info, AirspacePermit>,

    /// The identity account granting issuer permission for the authority.
    ///
    /// This account is not always required to exist, and only required when the airspace
    /// is restricted, and the authority is not the airspace authority.
    #[account(seeds = [
                AIRSPACE_PERMIT_ISSUER,
                airspace.key().as_ref(),
                authority.key().as_ref()
             ],
             bump
    )]
    issuer_id: AccountInfo<'info>,

    system_program: Program<'info, System>,
}

pub fn airspace_permit_create_handler(
    ctx: Context<AirspacePermitCreate>,
    owner: Pubkey,
) -> Result<()> {
    // First validate that the signer is allowed to create permits

    let airspace = &ctx.accounts.airspace;
    let authority = &ctx.accounts.authority;

    // If the airspace is not restricted, then any signer can create permits
    if airspace.is_restricted && airspace.authority != authority.key() {
        // For a restricted airspace, the optional regulator account needs to be verified
        // to prove that the signer is authorized to create the permit
        let _ = Account::<AirspacePermitIssuerId>::try_from(&ctx.accounts.issuer_id)?;

        // No further checks are necessary, since the address is already verified by anchor,
        // and the account data being valid to deserialize means the permission was granted
    }

    let permit = &mut ctx.accounts.permit;

    permit.airspace = ctx.accounts.airspace.key();
    permit.owner = owner;
    permit.issuer = if airspace.authority == authority.key() {
        airspace.key()
    } else {
        authority.key()
    };

    emit!(AirspacePermitCreated {
        airspace: airspace.key(),
        issuer: permit.issuer,
        owner: permit.owner
    });

    Ok(())
}
