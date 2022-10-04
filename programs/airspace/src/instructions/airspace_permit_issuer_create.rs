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
    events::AirspaceIssuerIdCreated,
    seeds::AIRSPACE_PERMIT_ISSUER,
    state::{Airspace, AirspacePermitIssuerId},
};

#[derive(Accounts)]
#[instruction(issuer: Pubkey)]
pub struct AirspacePermitIssuerCreate<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    /// The airspace authority
    authority: Signer<'info>,

    /// The airspace the regulator will grant permits for
    airspace: Account<'info, Airspace>,

    /// The license account, which will prove the given regulator has authority to
    /// grant new permits.
    #[account(init,
              seeds = [
                AIRSPACE_PERMIT_ISSUER,
                airspace.key().as_ref(),
                issuer.as_ref()
              ],
              bump,
              payer = payer,
              space = AirspacePermitIssuerId::SIZE
    )]
    issuer_id: Account<'info, AirspacePermitIssuerId>,

    system_program: Program<'info, System>,
}

pub fn airspace_permit_issuer_create_handler(
    ctx: Context<AirspacePermitIssuerCreate>,
    issuer: Pubkey,
) -> Result<()> {
    let airspace = &ctx.accounts.airspace;
    let issuer_id = &mut ctx.accounts.issuer_id;

    issuer_id.airspace = airspace.key();
    issuer_id.issuer = issuer;

    emit!(AirspaceIssuerIdCreated {
        airspace: airspace.key(),
        issuer
    });

    Ok(())
}
