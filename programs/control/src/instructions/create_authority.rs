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

use crate::events;

#[derive(Accounts)]
pub struct CreateAuthority<'info> {
    #[account(
        init,
        seeds = [],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<Authority>(),
    )]
    authority: Account<'info, Authority>,

    #[account(mut)]
    payer: Signer<'info>,

    system_program: Program<'info, System>,
}

#[account]
#[derive(Default)]
pub struct Authority {
    pub seed: [u8; 1],
}

pub fn create_authority_handler(ctx: Context<CreateAuthority>) -> Result<()> {
    ctx.accounts.authority.seed[0] = *ctx.bumps.get("authority").unwrap();

    emit!(events::AuthorityCreated {
        authority: ctx.accounts.authority.key(),
        seed: ctx.accounts.authority.seed[0],
    });

    Ok(())
}
