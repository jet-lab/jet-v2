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
    events::AirspacePermitRevoked,
    seeds::AIRSPACE_PERMIT_ISSUER,
    state::{Airspace, AirspacePermit},
    AirspaceErrorCode,
};

#[derive(Accounts)]
pub struct AirspacePermitRevoke<'info> {
    #[account(mut)]
    receiver: Signer<'info>,

    /// The authority allowed to revoke an airspace permit
    ///
    /// The addresses allowed to revoke are:
    ///     * the airspace authority, always
    ///     * the regulator that issued the permit, always
    ///     * any address, if the airspace is restricted and the regulator license
    ///       has been revoked
    /// The only addresses that can revoke a permit are either the regulator that
    /// created the permit, or the airspace authority.
    authority: Signer<'info>,

    /// The airspace the permit is to be revoked from
    airspace: Account<'info, Airspace>,

    /// The identity account for the regulator that issued the permit
    #[account(seeds = [
                AIRSPACE_PERMIT_ISSUER,
                airspace.key().as_ref(),
                permit.issuer.as_ref()
              ],
              bump
    )]
    issuer_id: AccountInfo<'info>,

    /// The airspace account to be created
    #[account(mut,
              close = receiver,
              has_one = airspace
    )]
    permit: Account<'info, AirspacePermit>,
}

pub fn airspace_permit_revoke_handler(ctx: Context<AirspacePermitRevoke>) -> Result<()> {
    let airspace = &mut ctx.accounts.airspace;
    let permit = &ctx.accounts.permit;
    let authority = ctx.accounts.authority.key();

    // The airspace authority or issuing regulator is always allowed to revoke
    if authority != airspace.authority && authority != permit.issuer {
        return err!(AirspaceErrorCode::PermissionDenied);
    }

    // For restricted airspaces, anyone can revoke a permit from a revoked regulator.
    // For unrestricted airspaces, permits cannot be revoked
    if !airspace.is_restricted
        || !ctx.accounts.issuer_id.data_is_empty()
        || permit.issuer == airspace.key()
    {
        return err!(AirspaceErrorCode::PermissionDenied);
    }

    emit!(AirspacePermitRevoked {
        airspace: airspace.key(),
        issuer: permit.issuer,
        owner: permit.owner
    });

    Ok(())
}
