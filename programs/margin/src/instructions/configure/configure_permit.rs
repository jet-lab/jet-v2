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

use anchor_lang::{prelude::*, AccountsClose};

use jet_airspace::state::Airspace;

use crate::{seeds::PERMIT_SEED, Permissions, Permit, StorageSpace};

#[derive(Accounts)]
pub struct ConfigurePermit<'info> {
    /// The authority allowed to make changes to configuration
    pub authority: Signer<'info>,

    /// The airspace being modified
    #[account(has_one = authority)]
    pub airspace: Account<'info, Airspace>,

    /// The payer for any rent costs, if required
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The owner being configured
    pub owner: AccountInfo<'info>,

    /// The config account to be modified
    #[account(init_if_needed,
              seeds = [
                PERMIT_SEED,
                airspace.key().as_ref(),
                owner.key().as_ref()
              ],
              bump,
              payer = payer,
              space = Permit::SPACE,
    )]
    pub permit: Account<'info, Permit>,

    pub system_program: Program<'info, System>,
}

pub fn configure_permit(
    ctx: Context<ConfigurePermit>,
    test: bool,
    flag: Permissions,
) -> Result<()> {
    let permit = &mut ctx.accounts.permit;

    permit.owner = ctx.accounts.owner.key();
    permit.airspace = ctx.accounts.airspace.key();

    if test {
        permit.permissions |= flag;
    } else {
        permit.permissions.remove(flag);
    }

    if permit.permissions.is_empty() {
        return permit.close(ctx.accounts.payer.to_account_info());
    }

    Ok(())
}
