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

use std::num::NonZeroU64;

use anchor_spl::dex;
use anchor_spl::dex::serum_dex::matching::{OrderType, Side};
use anchor_spl::dex::serum_dex::{instruction::SelfTradeBehavior, state::OpenOrders};
use anchor_spl::token::Token;

use crate::*;

#[derive(Accounts)]
pub struct SerumSwap<'info> {
    /// The margin account being executed on
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The account with the source deposit to be exchanged from
    /// CHECK:
    #[account(mut)]
    pub source_account: AccountInfo<'info>,

    /// The destination account to send the deposit that is exchanged into
    /// CHECK:
    #[account(mut)]
    pub destination_account: AccountInfo<'info>,

    /// Temporary SPL account to send tokens
    /// CHECK:
    #[account(mut, constraint = transit_source_account.owner == &margin_account.key())]
    pub transit_source_account: AccountInfo<'info>,

    /// Temporary SPL account to receive tokens
    /// CHECK:
    #[account(mut, constraint = transit_destination_account.owner == &margin_account.key())]
    pub transit_destination_account: AccountInfo<'info>,

    /// The accounts relevant to the swap pool used for the exchange
    pub swap_info: SerumSwapInfo<'info>,

    /// The accounts relevant to the source margin pool
    pub source_margin_pool: MarginPoolInfo<'info>,

    /// The accounts relevant to the destination margin pool
    pub destination_margin_pool: MarginPoolInfo<'info>,

    pub margin_pool_program: Program<'info, JetMarginPool>,

    pub token_program: Program<'info, Token>,

    pub rent: Sysvar<'info, Rent>,
}

impl<'info> SerumSwap<'info> {
    fn withdraw_source_context(&self) -> CpiContext<'_, '_, '_, 'info, Withdraw<'info>> {
        CpiContext::new(
            self.margin_pool_program.to_account_info(),
            Withdraw {
                margin_pool: self.source_margin_pool.margin_pool.to_account_info(),
                vault: self.source_margin_pool.vault.to_account_info(),
                deposit_note_mint: self.source_margin_pool.deposit_note_mint.to_account_info(),
                depositor: self.margin_account.to_account_info(),
                source: self.source_account.to_account_info(),
                destination: self.transit_source_account.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn new_order_v3_context(&self) -> CpiContext<'_, '_, '_, 'info, dex::NewOrderV3<'info>> {
        CpiContext::new(
            self.swap_info.serum_program.to_account_info(),
            dex::NewOrderV3 {
                market: self.swap_info.market.to_account_info(),
                open_orders: self.swap_info.open_orders.to_account_info(),
                request_queue: self.swap_info.request_queue.to_account_info(),
                event_queue: self.swap_info.event_queue.to_account_info(),
                market_bids: self.swap_info.market_bids.to_account_info(),
                market_asks: self.swap_info.market_asks.to_account_info(),
                order_payer_token_account: self.transit_source_account.to_account_info(),
                open_orders_authority: self.swap_info.open_orders_authority.to_account_info(),
                coin_vault: self.swap_info.base_vault.to_account_info(),
                pc_vault: self.swap_info.quote_vault.to_account_info(),
                token_program: self.token_program.to_account_info(),
                rent: self.rent.to_account_info(),
            },
        )
    }

    fn settle_funds_context(&self) -> CpiContext<'_, '_, '_, 'info, dex::SettleFunds<'info>> {
        CpiContext::new(
            self.swap_info.serum_program.to_account_info(),
            dex::SettleFunds {
                market: self.swap_info.market.to_account_info(),
                open_orders: self.swap_info.open_orders.to_account_info(),
                open_orders_authority: self.swap_info.open_orders_authority.to_account_info(),
                coin_vault: self.swap_info.base_vault.to_account_info(),
                pc_vault: self.swap_info.quote_vault.to_account_info(),
                // TODO: does the order of coin and pc depend on swap direction?
                coin_wallet: self.transit_source_account.to_account_info(),
                pc_wallet: self.transit_destination_account.to_account_info(),
                vault_signer: self.swap_info.vault_signer.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn deposit_destination_context(&self) -> CpiContext<'_, '_, '_, 'info, Deposit<'info>> {
        CpiContext::new(
            self.margin_pool_program.to_account_info(),
            Deposit {
                margin_pool: self.destination_margin_pool.margin_pool.to_account_info(),
                vault: self.destination_margin_pool.vault.to_account_info(),
                deposit_note_mint: self
                    .destination_margin_pool
                    .deposit_note_mint
                    .to_account_info(),
                depositor: self.margin_account.to_account_info(),
                source: self.transit_destination_account.to_account_info(),
                destination: self.destination_account.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct SerumSwapInfo<'info> {
    /// The Serum market to place the swap in
    /// CHECK:
    pub market: UncheckedAccount<'info>,

    /// CHECK:
    pub authority: UncheckedAccount<'info>,

    /// CHECK:
    pub open_orders: UncheckedAccount<'info>,

    /// This would be the margin account, however the liquidator can also
    /// invoke this instruction, and it would use its own open orders accounts
    /// per market, and thus should be able to sign as the authority.
    /// CHECK:
    pub open_orders_authority: UncheckedAccount<'info>,

    /// CHECK:
    pub base_vault: UncheckedAccount<'info>,

    /// CHECK:
    pub quote_vault: UncheckedAccount<'info>,

    /// CHECK:
    pub request_queue: UncheckedAccount<'info>,

    /// CHECK:
    pub event_queue: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub market_bids: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub market_asks: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub token_mint: UncheckedAccount<'info>,

    /// CHECK:
    pub vault_signer: AccountInfo<'info>,

    pub serum_program: Program<'info, dex::Dex>,
}

pub fn serum_swap_handler(
    ctx: Context<SerumSwap>,
    amount_in: u64,
    minimum_amount_out: u64,
    // TODO: will change this to an enum, quicker to bool it for now
    wants_base: bool,
) -> Result<()> {
    jet_margin_pool::cpi::withdraw(
        ctx.accounts.withdraw_source_context(),
        Amount::tokens(amount_in),
    )?;

    // Build order parameters
    let (side, limit_price, max_coin_qty, max_native_pc_qty) = if wants_base {
        let max_coin_qty = NonZeroU64::new(0).unwrap();
        let max_native_pc_qty = NonZeroU64::new(0).unwrap();
        (
            Side::Bid,
            NonZeroU64::new(u64::MAX).unwrap(),
            max_coin_qty,
            max_native_pc_qty,
        )
    } else {
        // TODO
        let max_coin_qty = NonZeroU64::new(0).unwrap();
        let max_native_pc_qty = NonZeroU64::new(u64::MAX).unwrap();
        (
            Side::Ask,
            NonZeroU64::new(1).unwrap(),
            max_coin_qty,
            max_native_pc_qty,
        )
    };

    dex::new_order_v3(
        ctx.accounts.new_order_v3_context(),
        side,
        limit_price,
        max_coin_qty,
        max_native_pc_qty,
        SelfTradeBehavior::DecrementTake,
        OrderType::ImmediateOrCancel,
        // We do not need to cancel orders, so a static ID is fine
        0,
        u16::MAX,
    )?;

    // Settle funds
    dex::settle_funds(ctx.accounts.settle_funds_context())?;

    // TODO: check if slippage is tolerable

    let destination_amount = token::accessor::amount(&ctx.accounts.transit_destination_account)?;

    jet_margin_pool::cpi::deposit(
        ctx.accounts.deposit_destination_context(),
        destination_amount,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct InitOpenOrders<'info> {
    /// The owner of the open orders account, expected to be the margin account
    /// or a liquidator.
    #[account(signer)]
    pub owner: AccountLoader<'info, MarginAccount>,

    /// CHECK: The account is validated by `serum_dex::init_open_orders`
    #[account(mut)]
    pub market: AccountInfo<'info>,

    /// The address paying for rent
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: New account that is checked and owned by Serum
    #[account(
        init,
        seeds = [
            owner.key().as_ref(),
            market.key().as_ref(),
            b"open_orders",
        ],
        bump,
        payer = payer,
        owner = dex::Dex::id(),
        space = 12 + std::mem::size_of::<OpenOrders>(), // serum padding = 12
    )]
    pub open_orders: AccountInfo<'info>,

    pub serum_program: Program<'info, dex::Dex>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitOpenOrders<'info> {
    fn init_open_oders_context(&self) -> CpiContext<'_, '_, '_, 'info, dex::InitOpenOrders<'info>> {
        CpiContext::new(
            self.serum_program.to_account_info(),
            dex::InitOpenOrders {
                open_orders: self.open_orders.to_account_info(),
                authority: self.owner.to_account_info(),
                market: self.market.to_account_info(),
                rent: self.rent.to_account_info(),
            },
        )
    }
}

pub fn init_open_orders_handler(ctx: Context<InitOpenOrders>) -> Result<()> {
    dex::init_open_orders(ctx.accounts.init_open_oders_context())
}

#[derive(Accounts)]
pub struct CloseOpenOrders<'info> {
    /// The owner of the open orders account, expected to be the margin account
    /// or a liquidator.
    #[account(signer)]
    pub owner: AccountLoader<'info, MarginAccount>,

    /// CHECK: New account that is checked and owned by Serum
    #[account(
        seeds = [
            owner.key().as_ref(),
            market.key().as_ref(),
            b"open_orders",
        ],
        bump,
    )]
    pub open_orders: AccountInfo<'info>,

    /// CHECK: The account is validated by `serum_dex::init_open_orders`
    #[account(mut)]
    pub market: AccountInfo<'info>,

    /// The destination account to send SOL to
    /// CHECK: Account only needs to be able to receive SOL
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,

    pub serum_program: Program<'info, dex::Dex>,
}

impl<'info> CloseOpenOrders<'info> {
    fn close_open_oders_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, dex::CloseOpenOrders<'info>> {
        CpiContext::new(
            self.serum_program.to_account_info(),
            dex::CloseOpenOrders {
                open_orders: self.open_orders.to_account_info(),
                authority: self.owner.to_account_info(),
                market: self.market.to_account_info(),
                destination: self.destination.to_account_info(),
            },
        )
    }
}

pub fn close_open_orders_handler(ctx: Context<CloseOpenOrders>) -> Result<()> {
    dex::close_open_orders(ctx.accounts.close_open_oders_context())
}
