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

use crate::{seeds::ADAPTER_CONFIG_SEED, AdapterConfig};

#[derive(Accounts)]
pub struct ConfigureAdapter<'info> {
    /// The authority allowed to make changes to configuration
    pub authority: Signer<'info>,

    /// The airspace being modified
    #[account(has_one = authority)]
    pub airspace: Account<'info, Airspace>,

    /// The payer for any rent costs, if required
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The adapter being configured
    pub adapter_program: AccountInfo<'info>,

    /// The config account to be modified
    #[account(init_if_needed,
              seeds = [
                ADAPTER_CONFIG_SEED,
                airspace.key().as_ref(),
                adapter_program.key().as_ref()
              ],
              bump,
              payer = payer,
              space = AdapterConfig::SPACE,
    )]
    pub adapter_config: Account<'info, AdapterConfig>,

    pub system_program: Program<'info, System>,
}

pub fn configure_adapter_handler(ctx: Context<ConfigureAdapter>, is_adapter: bool) -> Result<()> {
    let config = &mut ctx.accounts.adapter_config;

    if !is_adapter {
        return config.close(ctx.accounts.payer.to_account_info());
    };

    config.adapter_program = ctx.accounts.adapter_program.key();
    config.airspace = ctx.accounts.airspace.key();

    Ok(())
}
