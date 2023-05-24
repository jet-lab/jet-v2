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
use orca_whirlpool::cpi::accounts::InitializeConfig;

use crate::seeds::ORCA_WHIRLPOOL_CONFIG;

#[derive(Accounts)]
pub struct OrcaWhirlpoolCreateConfig<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    #[account(mut,
              seeds = [ORCA_WHIRLPOOL_CONFIG],
              bump,
    )]
    config: AccountInfo<'info>,

    whirlpool_program: AccountInfo<'info>,
    system_program: Program<'info, System>,
}

pub fn orca_whirlpool_create_config_handler(
    ctx: Context<OrcaWhirlpoolCreateConfig>,
    authority: Pubkey,
    default_fee_rate: u16,
) -> Result<()> {
    orca_whirlpool::cpi::initialize_config(
        CpiContext::new_with_signer(
            ctx.accounts.whirlpool_program.to_account_info(),
            InitializeConfig {
                funder: ctx.accounts.payer.to_account_info(),
                config: ctx.accounts.config.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
            },
            &[&[ORCA_WHIRLPOOL_CONFIG, &[*ctx.bumps.get("config").unwrap()]]],
        ),
        authority,
        authority,
        authority,
        default_fee_rate,
    )?;

    Ok(())
}
