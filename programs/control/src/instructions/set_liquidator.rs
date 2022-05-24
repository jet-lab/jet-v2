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
use std::convert::TryInto;

use jet_metadata::cpi::accounts::{CreateEntry, RemoveEntry, SetEntry};
use jet_metadata::program::JetMetadata;
use jet_metadata::LiquidatorMetadata;

use super::Authority;

#[derive(Accounts)]
pub struct SetLiquidator<'info> {
    #[cfg_attr(not(feature = "devnet"), account(address = crate::ROOT_AUTHORITY))]
    pub requester: Signer<'info>,
    pub authority: Account<'info, Authority>,

    /// CHECK:
    pub liquidator: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub metadata_account: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub metadata_program: Program<'info, JetMetadata>,
    pub system_program: Program<'info, System>,
}

impl<'info> SetLiquidator<'info> {
    fn create_metadata_context(&self) -> CpiContext<'_, '_, '_, 'info, CreateEntry<'info>> {
        CpiContext::new(
            self.metadata_program.to_account_info(),
            CreateEntry {
                key_account: self.liquidator.to_account_info(),
                metadata_account: self.metadata_account.to_account_info(),
                authority: self.authority.to_account_info(),

                payer: self.payer.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        )
    }

    fn set_metadata_context(&self) -> CpiContext<'_, '_, '_, 'info, SetEntry<'info>> {
        CpiContext::new(
            self.metadata_program.to_account_info(),
            SetEntry {
                metadata_account: self.metadata_account.to_account_info(),
                authority: self.authority.to_account_info(),
            },
        )
    }

    fn remove_metadata_context(&self) -> CpiContext<'_, '_, '_, 'info, RemoveEntry<'info>> {
        CpiContext::new(
            self.metadata_program.to_account_info(),
            RemoveEntry {
                metadata_account: self.metadata_account.to_account_info(),
                authority: self.authority.to_account_info(),
                receiver: self.payer.to_account_info(),
            },
        )
    }
}

pub fn set_liquidator_handler(ctx: Context<SetLiquidator>, is_liquidator: bool) -> Result<()> {
    let authority = [&ctx.accounts.authority.seed[..]];

    let mut data = vec![];
    let metadata = LiquidatorMetadata {
        liquidator: ctx.accounts.liquidator.key(),
    };

    metadata.try_serialize(&mut data)?;

    if is_liquidator {
        jet_metadata::cpi::create_entry(
            ctx.accounts
                .create_metadata_context()
                .with_signer(&[&authority]),
            String::new(),
            data.len().try_into().unwrap(),
        )?;
        jet_metadata::cpi::set_entry(
            ctx.accounts
                .set_metadata_context()
                .with_signer(&[&authority]),
            0,
            data,
        )?;
    } else {
        jet_metadata::cpi::remove_entry(
            ctx.accounts
                .remove_metadata_context()
                .with_signer(&[&authority]),
        )?;
    }

    Ok(())
}
