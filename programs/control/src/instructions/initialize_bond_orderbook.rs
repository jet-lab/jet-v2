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
use jet_bonds::program::JetBonds;

use super::Authority;

#[derive(Accounts)]
pub struct InitializeBondOrderbook<'info> {
    #[cfg_attr(not(feature = "testing"), account(address = crate::ROOT_AUTHORITY))]
    #[account(mut)]
    requester: Signer<'info>,
    authority: Account<'info, Authority>,

    pub initialization_accounts: InitializeOrderbook<'info>,
    pub bonds_program: Program<'info, JetBonds>,
}

#[derive(Accounts)]
pub struct InitializeOrderbook<'info> {
    /// The `BondManager` account tracks global information related to this particular bond market
    /// CHECK:
    pub bond_manager: AccountInfo<'info>,

    /// Accounts for `agnostic-orderbook`
    /// Should be uninitialized, used for invoking create_account and sent to the agnostic orderbook program
    /// CHECK: handled by aaob
    pub orderbook_market_state: AccountInfo<'info>,
    /// CHECK: handled by aaob
    pub event_queue: AccountInfo<'info>,
    /// CHECK: handled by aaob
    pub bids: AccountInfo<'info>,
    /// CHECK: handled by aaob
    pub asks: AccountInfo<'info>,

    /// The authority to create markets, which must sign
    /// CHECK:
    pub program_authority: AccountInfo<'info>,

    /// The account paying rent for PDA initialization
    /// CHECK:
    pub payer: AccountInfo<'info>,

    /// Solana system program
    /// CHECK:
    pub system_program: AccountInfo<'info>,
}

#[inline(never)]
pub fn initialize_bond_orderbook_handler(
    ctx: Context<InitializeBondOrderbook>,
    params: jet_bonds::control::instructions::InitializeOrderbookParams,
) -> Result<()> {
    let authority = [&ctx.accounts.authority.seed[..]];

    let cpi_accounts = {
        let accs = &mut ctx.accounts.initialization_accounts;
        jet_bonds::cpi::accounts::InitializeOrderbook {
            bond_manager: accs.bond_manager.to_account_info(),
            orderbook_market_state: accs.orderbook_market_state.to_account_info(),
            event_queue: accs.event_queue.to_account_info(),
            bids: accs.bids.to_account_info(),
            asks: accs.asks.to_account_info(),
            program_authority: accs.program_authority.to_account_info(),
            payer: accs.payer.to_account_info(),
            system_program: accs.system_program.to_account_info(),
        }
    };

    // initialize the market
    jet_bonds::cpi::initialize_orderbook(
        CpiContext::new(ctx.accounts.bonds_program.to_account_info(), cpi_accounts)
            .with_signer(&[&authority]),
        params,
    )?;

    Ok(())
}
