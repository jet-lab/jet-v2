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

use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::dex;
use anchor_spl::dex::serum_dex::instruction::SelfTradeBehavior;
use anchor_spl::dex::serum_dex::matching::{OrderType, Side};
use anchor_spl::token::{Mint, Token, TokenAccount};
use jet_program_common::Number128;

use crate::instructions::utils::read_price;
use crate::seeds::SWAP_POOL_TOKENS;
use crate::state::OpenBookMarketInfo;

#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone, Eq, PartialEq)]
pub struct OpenBookMarketMakeParams {
    pub bid_from_order_id: u64,
    pub ask_from_order_id: u64,
}

#[derive(Accounts)]
pub struct OpenBookMarketMake<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    #[account(mut)]
    open_orders_owner: Signer<'info>,

    #[account(mut)]
    mint_base: Box<Account<'info, Mint>>,

    #[account(mut)]
    mint_quote: Box<Account<'info, Mint>>,

    #[account(mut,
        seeds = [
          SWAP_POOL_TOKENS,
          market_state.key().as_ref(),
          mint_base.key().as_ref()
        ],
        bump
    )]
    vault_base: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        seeds = [
          SWAP_POOL_TOKENS,
          market_state.key().as_ref(),
          mint_quote.key().as_ref()
        ],
        bump,
    )]
    vault_quote: Box<Account<'info, TokenAccount>>,

    #[account(mut, constraint = wallet_base.mint == mint_base.key())]
    wallet_base: Box<Account<'info, TokenAccount>>,

    #[account(mut, constraint = wallet_quote.mint == mint_quote.key())]
    wallet_quote: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    market_info: Box<Account<'info, OpenBookMarketInfo>>,

    #[account(mut,
        seeds = [
          b"openbook-market", // TODO: consts
          mint_base.key().as_ref(),
          mint_quote.key().as_ref(),
        ],
        bump
    )]
    market_state: AccountInfo<'info>,

    #[account(mut)]
    bids: AccountInfo<'info>,
    #[account(mut)]
    asks: AccountInfo<'info>,
    #[account(mut)]
    request_queue: AccountInfo<'info>,
    #[account(mut)]
    event_queue: AccountInfo<'info>,

    #[account(mut)]
    open_orders: AccountInfo<'info>,

    pyth_price_base: AccountInfo<'info>,
    pyth_price_quote: AccountInfo<'info>,

    dex_program: AccountInfo<'info>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

impl<'info> OpenBookMarketMake<'info> {
    fn create_order(
        &self,
        signer_seeds: &[&[&[u8]]],
        side: Side,
        limit_price: NonZeroU64,
        max_coin_qty: NonZeroU64,
        max_native_pc_qty_including_fees: NonZeroU64,
        client_order_id: u64,
    ) -> Result<()> {
        let new_order_context = CpiContext::new_with_signer(
            self.dex_program.to_account_info(),
            dex::NewOrderV3 {
                market: self.market_state.to_account_info(),
                coin_vault: self.vault_base.to_account_info(),
                pc_vault: self.vault_quote.to_account_info(),
                market_bids: self.bids.to_account_info(),
                market_asks: self.asks.to_account_info(),
                rent: self.rent.to_account_info(),
                open_orders: self.open_orders.to_account_info(),
                request_queue: self.request_queue.to_account_info(),
                event_queue: self.event_queue.to_account_info(),
                order_payer_token_account: match side {
                    Side::Bid => self.wallet_quote.to_account_info(),
                    Side::Ask => self.wallet_base.to_account_info(),
                },
                open_orders_authority: self.open_orders_owner.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
            signer_seeds,
        );

        dex::new_order_v3(
            new_order_context,
            side,
            limit_price,
            max_coin_qty,
            max_native_pc_qty_including_fees,
            SelfTradeBehavior::AbortTransaction,
            OrderType::Limit,
            client_order_id,
            u16::MAX,
        )
    }

    fn cancel_orders(&self, signer_seeds: &[&[&[u8]]], order_ids: [u64; 8]) -> Result<()> {
        let cancel_orders_ix = openbook::instruction::cancel_orders_by_client_order_ids(
            self.dex_program.key,
            self.market_state.key,
            self.bids.key,
            self.asks.key,
            self.open_orders.key,
            self.open_orders_owner.key,
            self.event_queue.key,
            order_ids,
        )
        .unwrap();

        invoke_signed(
            &cancel_orders_ix,
            &[
                self.market_state.to_account_info(),
                self.bids.to_account_info(),
                self.asks.to_account_info(),
                self.open_orders.to_account_info(),
                self.open_orders_owner.to_account_info(),
                self.event_queue.to_account_info(),
            ],
            signer_seeds,
        )
        .map_err(|e| e.into())
    }
}

pub fn openbook_market_make_handler(
    ctx: Context<OpenBookMarketMake>,
    params: OpenBookMarketMakeParams,
) -> Result<()> {
    let bump = *ctx.bumps.get("market_state").unwrap();

    let mint_base_key = ctx.accounts.mint_base.key();
    let mint_quote_key = ctx.accounts.mint_quote.key();

    let market_signer_seeds = [
        b"openbook-market",
        mint_base_key.as_ref(),
        mint_quote_key.as_ref(),
        &[bump],
    ];
    let seeds = [&market_signer_seeds[..]];

    let market_info = &ctx.accounts.market_info;

    let bid_order_ids: [u64; 8] = (params.bid_from_order_id..)
        .take(8)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let ask_order_ids: [u64; 8] = (params.ask_from_order_id..)
        .take(8)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    ctx.accounts.cancel_orders(&seeds, bid_order_ids)?;
    ctx.accounts.cancel_orders(&seeds, ask_order_ids)?;
    // Create new orders of equal size on both sides of the book, with some incremental spread
    let price_base = read_price(&ctx.accounts.pyth_price_base);
    let price_quote = read_price(&ctx.accounts.pyth_price_quote);

    let market_price = price_base / price_quote;
    msg!("Current market price is {:?}", market_price);

    let base_lamports = ctx.accounts.mint_base.decimals as i32;
    let quote_lamports = ctx.accounts.mint_quote.decimals as i32;

    let (base_lot_size, quote_lot_size) = {
        let market_account = ctx.accounts.market_state.to_account_info();
        let market = openbook::state::MarketState::load(
            &market_account,
            ctx.accounts.dex_program.key,
            false,
        )
        .unwrap();
        ({ market.coin_lot_size }, { market.pc_lot_size })
    };

    // Start with 1% from market price
    let mut bid_price = market_price * Number128::from_bps(10000 - market_info.initial_spread);
    let mut ask_price = market_price * Number128::from_bps(10100 + market_info.initial_spread);
    let bid_spread_increment = Number128::from_bps(10000 - market_info.incremental_spread);
    let ask_spread_increment = Number128::from_bps(10000 + market_info.incremental_spread);

    let baskets = market_info.basket_sizes;
    let basket_liquidity = market_info.basket_liquidity;
    let basket_liquidity_usd = Number128::from_decimal(basket_liquidity, 0);

    // Bids
    for (order_id, basket_size) in bid_order_ids.into_iter().zip(baskets) {
        let bid_price_tokens = bid_price.as_u64(-quote_lamports);
        let limit_price = price_number_to_lot(
            bid_price_tokens,
            base_lamports as _,
            base_lot_size,
            quote_lot_size,
        );
        ctx.accounts.create_order(
            &seeds,
            Side::Bid,
            NonZeroU64::new(limit_price).unwrap(),
            NonZeroU64::new(u64::MAX).unwrap(),
            NonZeroU64::new(
                (basket_liquidity * 10u64.pow(quote_lamports as u32))
                    .saturating_mul(basket_size as _),
            )
            .unwrap(),
            order_id,
        )?;
        bid_price = bid_price * bid_spread_increment;
        println!("New bid price: {}", bid_price);
    }
    // Asks
    for (order_id, basket_size) in ask_order_ids.into_iter().zip(baskets) {
        let ask_price_tokens = ask_price.as_u64(-quote_lamports);
        let limit_price = price_number_to_lot(
            ask_price_tokens,
            base_lamports as _,
            base_lot_size,
            quote_lot_size,
        );
        let base_quantity = (basket_liquidity_usd / ask_price)
            .as_u64(-base_lamports)
            .saturating_mul(basket_size as _);
        ctx.accounts.create_order(
            &seeds,
            Side::Ask,
            NonZeroU64::new(limit_price).unwrap(),
            NonZeroU64::new(base_quantity.saturating_div(base_lot_size)).unwrap(),
            NonZeroU64::new(u64::MAX).unwrap(),
            order_id,
        )?;
        ask_price = ask_price * ask_spread_increment;
        println!("New ask price: {}", ask_price);
    }

    Ok(())
}

/// Convert a price from quote tokens to lot sizes.
///
/// A USDC price of 1 will have 1_000_000 tokens as it has 6 decimals.
fn price_number_to_lot(
    price: u64,
    base_lamports: u64,
    base_lot_size: u64,
    quote_lot_size: u64,
) -> u64 {
    price
        .saturating_mul(base_lot_size)
        .saturating_div(base_lamports.saturating_mul(quote_lot_size))
}
