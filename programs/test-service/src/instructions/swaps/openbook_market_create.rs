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
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::seeds::SWAP_POOL_TOKENS;
use crate::state::{OpenBookMarketInfo, TokenInfo};

#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone, Eq, PartialEq)]
pub struct OpenBookMarketCreateParams {
    pub vault_signer_nonce: u64,
    pub base_lot_size: u64,
    pub quote_lot_size: u64,
    pub quote_dust_threshold: u64,
    pub liquidity_amount: u64,
    pub initial_spread: u16,
    pub incremental_spread: u16,
    pub basket_sizes: [u8; 8],
}

#[derive(Accounts)]
pub struct OpenBookMarketCreate<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    #[account(mut)]
    mint_base: Box<Account<'info, Mint>>,

    #[account(mut)]
    mint_quote: Box<Account<'info, Mint>>,

    #[account(constraint = info_base.mint == mint_base.key())]
    info_base: Box<Account<'info, TokenInfo>>,

    #[account(constraint = info_quote.mint == mint_quote.key())]
    info_quote: Box<Account<'info, TokenInfo>>,

    #[account(init,
              seeds = [
                b"openbook-market-info",
                mint_base.key().as_ref(),
                mint_quote.key().as_ref(),
              ],
              bump,
              space = 8 + std::mem::size_of::<OpenBookMarketInfo>(),
              payer = payer
    )]
    market_info: Box<Account<'info, OpenBookMarketInfo>>,

    #[account(init,
              seeds = [
                b"openbook-market", // TODO: consts
                mint_base.key().as_ref(),
                mint_quote.key().as_ref(),
              ],
              bump,
              space = 12 + std::mem::size_of::<openbook::state::MarketState>(),
              owner = dex::ID,
              payer = payer
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

    vault_signer: AccountInfo<'info>,

    #[account(init,
              seeds = [
                SWAP_POOL_TOKENS,
                market_state.key().as_ref(),
                mint_base.key().as_ref()
              ],
              bump,
              token::mint = mint_base,
              token::authority = vault_signer,
              payer = payer,
    )]
    vault_base: Box<Account<'info, TokenAccount>>,

    #[account(init,
              seeds = [
                SWAP_POOL_TOKENS,
                market_state.key().as_ref(),
                mint_quote.key().as_ref()
              ],
              bump,
              token::mint = mint_quote,
              token::authority = vault_signer,
              payer = payer,
    )]
    vault_quote: Box<Account<'info, TokenAccount>>,

    // Created as part of this instruction as a convenience
    #[account(init,
              seeds = [
                b"openbook-open-orders",
                market_state.key().as_ref(),
                payer.key().as_ref()
              ],
              bump,
              space = 12 + std::mem::size_of::<openbook::state::OpenOrders>(),
              owner = dex::ID,
              payer = payer
    )]
    open_orders: AccountInfo<'info>,

    dex_program: AccountInfo<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

pub fn openbook_market_create_handler(
    ctx: Context<OpenBookMarketCreate>,
    params: OpenBookMarketCreateParams,
) -> Result<()> {
    let market_info = &mut ctx.accounts.market_info;
    market_info.market_state = ctx.accounts.market_state.key();
    market_info.basket_liquidity = params.liquidity_amount;
    market_info.initial_spread = params.initial_spread;
    market_info.incremental_spread = params.incremental_spread;
    market_info.basket_sizes = params.basket_sizes;

    let bump = *ctx.bumps.get("market_state").unwrap();

    let mint_base_key = ctx.accounts.mint_base.key();
    let mint_quote_key = ctx.accounts.mint_quote.key();

    let pool_signer_seeds = [
        b"openbook-market",
        mint_base_key.as_ref(),
        mint_quote_key.as_ref(),
        &[bump],
    ];
    let seeds = [&pool_signer_seeds[..]];

    let swap_context = CpiContext::new_with_signer(
        ctx.accounts.dex_program.to_account_info(),
        dex::InitializeMarket {
            market: ctx.accounts.market_state.to_account_info(),
            coin_mint: ctx.accounts.mint_base.to_account_info(),
            pc_mint: ctx.accounts.mint_quote.to_account_info(),
            coin_vault: ctx.accounts.vault_base.to_account_info(),
            pc_vault: ctx.accounts.vault_quote.to_account_info(),
            bids: ctx.accounts.bids.to_account_info(),
            asks: ctx.accounts.asks.to_account_info(),
            req_q: ctx.accounts.request_queue.to_account_info(),
            event_q: ctx.accounts.event_queue.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        },
        &seeds,
    );

    dex::initialize_market(
        swap_context,
        params.base_lot_size,
        params.quote_lot_size,
        params.vault_signer_nonce,
        params.quote_dust_threshold,
    )?;

    let open_orders_context = CpiContext::new_with_signer(
        ctx.accounts.dex_program.to_account_info(),
        dex::InitOpenOrders {
            open_orders: ctx.accounts.open_orders.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
            market: ctx.accounts.market_state.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        },
        &seeds,
    );

    dex::init_open_orders(open_orders_context)?;

    Ok(())
}
