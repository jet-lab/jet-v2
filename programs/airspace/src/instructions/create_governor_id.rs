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

use crate::{seeds::GOVERNOR_ID, state::GovernorId, GOVERNOR_DEFAULT};

#[derive(Accounts)]
pub struct CreateGovernorId<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    /// The governer identity account
    #[account(init_if_needed,
              seeds = [GOVERNOR_ID],
              bump,
              payer = payer,
              space = GovernorId::SIZE
    )]
    governor_id: Account<'info, GovernorId>,

    system_program: Program<'info, System>,
}

pub fn create_governor_id_handler(ctx: Context<CreateGovernorId>) -> Result<()> {
    let governor_id = &mut ctx.accounts.governor_id;

    if governor_id.governor == Pubkey::default() {
        if cfg!(feature = "testing") {
            // In testing environments, governor can be set by first caller
            governor_id.governor = ctx.accounts.payer.key();
        } else {
            // In production/mainnet, governor has hardcoded default
            governor_id.governor = GOVERNOR_DEFAULT;
        }
    }

    Ok(())
}
