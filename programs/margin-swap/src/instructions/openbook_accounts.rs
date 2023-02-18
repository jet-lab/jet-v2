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
use anchor_spl::dex;
use dex::serum_dex::state::OpenOrders;

use jet_margin::MarginAccount;

use crate::seeds::OPENBOOK_OPEN_ORDERS;

#[derive(Accounts)]
pub struct InitOpenOrders<'info> {
    /// The margin account with the position to close
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// CHECK: The account is validated by `serum_dex::init_open_orders`
    #[account(owner = dex::Dex::id())]
    pub market: AccountInfo<'info>,

    /// The address paying for rent
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: New account that is checked and owned by OpenBook
    #[account(
        init,
        seeds = [
            OPENBOOK_OPEN_ORDERS,
            margin_account.key().as_ref(),
            market.key().as_ref(),
        ],
        bump,
        payer = payer,
        owner = dex::Dex::id(),
        space = 12 + std::mem::size_of::<OpenOrders>(), // serum padding = 12
    )]
    pub open_orders: AccountInfo<'info>,

    pub dex_program: Program<'info, dex::Dex>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitOpenOrders<'info> {
    fn init_open_oders_context(&self) -> CpiContext<'_, '_, '_, 'info, dex::InitOpenOrders<'info>> {
        CpiContext::new(
            self.dex_program.to_account_info(),
            dex::InitOpenOrders {
                open_orders: self.open_orders.to_account_info(),
                authority: self.margin_account.to_account_info(),
                market: self.market.to_account_info(),
                rent: self.rent.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct CloseOpenOrders<'info> {
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// CHECK: The account will be validated by the dex program
    #[account(mut, owner = dex::Dex::id())]
    pub open_orders: AccountInfo<'info>,

    /// The destination account to send SOL to
    /// CHECK: Account only needs to be able to receive SOL
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,

    /// CHECK: The account will be validated by the dex program
    pub market: AccountInfo<'info>,

    pub dex_program: Program<'info, dex::Dex>,
}

impl<'info> CloseOpenOrders<'info> {
    fn close_open_orders_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, dex::CloseOpenOrders<'info>> {
        CpiContext::new(
            self.dex_program.to_account_info(),
            dex::CloseOpenOrders {
                open_orders: self.open_orders.to_account_info(),
                authority: self.margin_account.to_account_info(),
                destination: self.destination.to_account_info(),
                market: self.market.to_account_info(),
            },
        )
    }
}

pub fn init_open_orders_handler(ctx: Context<InitOpenOrders>) -> Result<()> {
    // Call init_open_orders
    dex::init_open_orders(ctx.accounts.init_open_oders_context())?;

    Ok(())
}

pub fn close_open_orders_handler(ctx: Context<CloseOpenOrders>) -> Result<()> {
    // Call close_open_orders
    dex::close_open_orders(ctx.accounts.close_open_orders_context())?;

    Ok(())
}
