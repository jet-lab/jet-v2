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
use lookup_table_registry::program::LookupTableRegistry;

use crate::{MarginAccount, SignerSeeds};

#[derive(Accounts)]
pub struct AppendToLookup<'info> {
    /// The authority that can register a lookup table for a margin account
    pub authority: Signer<'info>,

    /// The payer of the transaction
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The margin account to create this lookup account for
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The registry account
    #[account(mut)]
    pub registry_account: AccountInfo<'info>,

    /// The lookup table being created
    /// CHECK: the account will be validated by the lookup table program
    #[account(mut)]
    pub lookup_table: AccountInfo<'info>,

    /// CHECK: the account will be validated by the lookup table program
    pub address_lookup_table_program: AccountInfo<'info>,

    pub registry_program: Program<'info, LookupTableRegistry>,

    pub system_program: Program<'info, System>,
}

pub fn append_to_lookup_handler(
    ctx: Context<AppendToLookup>,
    discriminator: u64,
    addresses: Vec<Pubkey>,
) -> Result<()> {
    let account = ctx.accounts.margin_account.load()?;
    account.verify_authority(ctx.accounts.authority.key())?;

    let signer = account.signer_seeds_owned();
    drop(account);

    let append_ctx = CpiContext::new(
        ctx.accounts.registry_program.to_account_info(),
        lookup_table_registry::cpi::accounts::AppendToLookupTable {
            authority: ctx.accounts.margin_account.to_account_info(),
            payer: ctx.accounts.payer.to_account_info(),
            lookup_table: ctx.accounts.lookup_table.to_account_info(),
            registry_account: ctx.accounts.registry_account.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            address_lookup_table_program: ctx
                .accounts
                .address_lookup_table_program
                .to_account_info(),
        },
    );

    lookup_table_registry::cpi::append_to_lookup_table(
        append_ctx.with_signer(&[&signer.signer_seeds()]),
        addresses,
        discriminator,
    )?;
    Ok(())
}
