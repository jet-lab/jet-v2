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
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::dex;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::seeds::{OPENBOOK_MARKET, SWAP_POOL_TOKENS};
use crate::state::TokenInfo;

#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone, Eq, PartialEq)]
pub struct OpenBookMarketCancelOrdersParams {
    pub bid_from_order_id: u64,
    pub ask_from_order_id: u64,
}

#[derive(Accounts)]
pub struct OpenBookMarketCancelOrders<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    #[account(mut)]
    open_orders_owner: Signer<'info>,

    #[account(constraint = info_base.mint == mint_base.key())]
    info_base: Box<Account<'info, TokenInfo>>,

    #[account(constraint = info_quote.mint == mint_quote.key())]
    info_quote: Box<Account<'info, TokenInfo>>,

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

    #[account(mut,
        seeds = [
          OPENBOOK_MARKET,
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
    event_queue: AccountInfo<'info>,

    #[account(mut)]
    open_orders: AccountInfo<'info>,

    vault_signer: AccountInfo<'info>,

    dex_program: AccountInfo<'info>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

impl<'info> OpenBookMarketCancelOrders<'info> {
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

    fn settle_funds(&self, signer_seeds: &[&[&[u8]]]) -> Result<()> {
        let settle_funds_context = CpiContext::new_with_signer(
            self.dex_program.to_account_info(),
            dex::SettleFunds {
                market: self.market_state.to_account_info(),
                coin_vault: self.vault_base.to_account_info(),
                pc_vault: self.vault_quote.to_account_info(),
                open_orders: self.open_orders.to_account_info(),
                open_orders_authority: self.open_orders_owner.to_account_info(),
                token_program: self.token_program.to_account_info(),
                coin_wallet: self.wallet_base.to_account_info(),
                pc_wallet: self.wallet_quote.to_account_info(),
                vault_signer: self.vault_signer.to_account_info(),
            },
            signer_seeds,
        );

        dex::settle_funds(settle_funds_context)
    }
}

pub fn openbook_market_cancel_orders_handler(
    ctx: Context<OpenBookMarketCancelOrders>,
    params: OpenBookMarketCancelOrdersParams,
) -> Result<()> {
    let bump = *ctx.bumps.get("market_state").unwrap();

    let mint_base_key = ctx.accounts.mint_base.key();
    let mint_quote_key = ctx.accounts.mint_quote.key();

    let market_signer_seeds = [
        OPENBOOK_MARKET,
        mint_base_key.as_ref(),
        mint_quote_key.as_ref(),
        &[bump],
    ];
    let seeds = [&market_signer_seeds[..]];

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
    ctx.accounts.settle_funds(&seeds)
}
