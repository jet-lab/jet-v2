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
use jet_bonds::{control::instructions::InitializeBondManagerParams, program::JetBonds};

use super::Authority;

#[derive(Accounts)]
pub struct CreateBondMarket<'info> {
    #[cfg_attr(not(feature = "testing"), account(address = crate::ROOT_AUTHORITY))]
    #[account(mut)]
    requester: Signer<'info>,
    authority: Account<'info, Authority>,

    pub initialization_accounts: InitializeBondManager<'info>,
    pub bonds_program: Program<'info, JetBonds>,
}

#[derive(Accounts)]
pub struct InitializeBondManager<'info> {
    /// CHECK:
    pub bond_manager: AccountInfo<'info>,
    /// CHECK:
    pub underlying_token_vault: AccountInfo<'info>,
    /// CHECK:
    pub underlying_token_mint: AccountInfo<'info>,
    /// CHECK:
    pub bond_ticket_mint: AccountInfo<'info>,
    /// CHECK:
    pub claims: AccountInfo<'info>,
    /// CHECK:
    pub collateral: AccountInfo<'info>,
    /// CHECK:
    pub program_authority: AccountInfo<'info>,
    /// The oracle for the underlying asset price
    /// CHECK: determined by caller
    pub underlying_oracle: AccountInfo<'info>,
    /// The oracle for the bond ticket price
    /// CHECK: determined by caller
    pub ticket_oracle: AccountInfo<'info>,
    /// The account paying rent for PDA initialization
    /// CHECK:
    pub payer: AccountInfo<'info>,
    /// Rent sysvar
    /// CHECK:
    pub rent: AccountInfo<'info>,
    /// SPL token program
    /// CHECK:
    pub token_program: AccountInfo<'info>,
    /// Solana system program
    /// CHECK:
    pub system_program: AccountInfo<'info>,
}

#[inline(never)]
pub fn create_bond_market_handler(
    ctx: Context<CreateBondMarket>,
    params: InitializeBondManagerParams,
) -> Result<()> {
    let authority = [&ctx.accounts.authority.seed[..]];

    let cpi_accounts = {
        let accs = &mut ctx.accounts.initialization_accounts;
        jet_bonds::cpi::accounts::InitializeBondManager {
            bond_manager: accs.bond_manager.to_account_info(),
            underlying_token_vault: accs.underlying_token_vault.to_account_info(),
            underlying_token_mint: accs.underlying_token_mint.to_account_info(),
            bond_ticket_mint: accs.bond_ticket_mint.to_account_info(),
            claims: accs.claims.to_account_info(),
            collateral: accs.collateral.to_account_info(),
            program_authority: accs.program_authority.to_account_info(),
            underlying_oracle: accs.underlying_oracle.to_account_info(),
            ticket_oracle: accs.ticket_oracle.to_account_info(),
            payer: accs.payer.to_account_info(),
            rent: accs.rent.to_account_info(),
            token_program: accs.token_program.to_account_info(),
            system_program: accs.system_program.to_account_info(),
        }
    };

    // initialize the market
    jet_bonds::cpi::initialize_bond_manager(
        CpiContext::new(ctx.accounts.bonds_program.to_account_info(), cpi_accounts)
            .with_signer(&[&authority]),
        params,
    )?;

    Ok(())
}
