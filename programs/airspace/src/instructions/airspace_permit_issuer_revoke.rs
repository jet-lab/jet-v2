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
    events::AirspaceIssuerIdRevoked,
    state::{Airspace, AirspacePermitIssuerId},
};

#[derive(Accounts)]
pub struct AirspacePermitIssuerRevoke<'info> {
    #[account(mut)]
    receiver: Signer<'info>,

    /// The airspace authority
    authority: Signer<'info>,

    /// The airspace the regulator is to be removed from
    #[account(has_one = authority)]
    airspace: Account<'info, Airspace>,

    /// The license account that will be removed for the regulator
    #[account(mut,
              close = receiver,
              has_one = airspace
    )]
    issuer_id: Account<'info, AirspacePermitIssuerId>,
}

pub fn airspace_permit_issuer_revoke_handler(
    ctx: Context<AirspacePermitIssuerRevoke>,
) -> Result<()> {
    emit!(AirspaceIssuerIdRevoked {
        airspace: ctx.accounts.airspace.key(),
        issuer: ctx.accounts.issuer_id.issuer
    });

    Ok(())
}
