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

use crate::{MarginAccount, events};

#[derive(Accounts)]
#[instruction(seed: u16)]
pub struct CreateAccount<'info> {
    /// The owner of the new margin account
    pub owner: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// The margin account to initialize for the owner
    #[account(init,
              seeds = [owner.key.as_ref(), seed.to_le_bytes().as_ref()],
              bump,
              payer = payer,
              space = 8 + std::mem::size_of::<MarginAccount>(),
    )]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    pub system_program: Program<'info, System>,
}

pub fn create_account_handler(ctx: Context<CreateAccount>, seed: u16) -> Result<()> {
    let mut account = ctx.accounts.margin_account.load_init()?;

    account.initialize(
        *ctx.accounts.owner.key,
        seed,
        *ctx.bumps.get("margin_account").unwrap(),
    );
    emit!(events::AccountCreated {
        owner: ctx.accounts.owner.key(), 
        margin_account: ctx.accounts.margin_account.key(),
    });
    Ok(())
}
