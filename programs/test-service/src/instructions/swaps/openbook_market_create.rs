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

use std::collections::BTreeMap;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_pack::Pack;
use anchor_spl::dex;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::instructions::TokenRequest;
use crate::seeds::{
    SWAP_POOL_FEES, SWAP_POOL_INFO, SWAP_POOL_MINT, SWAP_POOL_STATE, SWAP_POOL_TOKENS,
};
use crate::state::{OpenBookMarketInfo, TokenInfo};

#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone, Eq, PartialEq)]
pub struct OpenBookMarketCreateParams {
    pub nonce: u8,
    pub base_lot_size: u64,
    pub quote_lot_size: u64,
    pub quote_dust_threshold: u64,
}

#[derive(Accounts)]
pub struct OpenBookMarketCreate<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    #[account(mut)]
    mint_base: Box<Account<'info, Mint>>,

    #[account(mut)]
    mint_quote: Box<Account<'info, Mint>>,

    #[account(constraint = info_mint.mint == mint_base.key())]
    info_base: Box<Account<'info, TokenInfo>>,

    #[account(constraint = info_quote.mint == mint_quote.key())]
    info_quote: Box<Account<'info, TokenInfo>>,

    #[account(init,
              seeds = [
                SWAP_POOL_INFO,
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
              space = 12 + std::mem::size_of::<dex::state::MarketState>(),
              owner = dex::ID,
              payer = payer
    )]
    market_state: AccountInfo<'info>,

    // TODO:
    #[account(mut)]
    bids: AccountInfo<'info>,
    #[account(mut)]
    asks: AccountInfo<'info>,
    #[account(mut)]
    request_queue: AccountInfo<'info>,
    #[account(mut)]
    event_queue: AccountInfo<'info>,

    #[account(seeds = [market_state.key().as_ref()],
              bump,
              seeds::program = dex::ID
    )]
    pool_authority: AccountInfo<'info>,

    #[account(init,
              seeds = [
                SWAP_POOL_MINT,
                market_state.key().as_ref()
              ],
              bump,
              mint::decimals = mint_base.decimals,
              mint::authority = pool_authority,
              payer = payer
    )]
    pool_mint: Box<Account<'info, Mint>>,

    #[account(init,
              seeds = [
                SWAP_POOL_TOKENS,
                market_state.key().as_ref(),
                mint_base.key().as_ref()
              ],
              bump,
              token::mint = mint_base,
              token::authority = pool_authority,
              payer = payer,
    )]
    pool_token_a: Box<Account<'info, TokenAccount>>,

    #[account(init,
              seeds = [
                SWAP_POOL_TOKENS,
                market_state.key().as_ref(),
                mint_quote.key().as_ref()
              ],
              bump,
              token::mint = mint_quote,
              token::authority = pool_authority,
              payer = payer,
    )]
    pool_token_b: Box<Account<'info, TokenAccount>>,

    #[account(init,
              seeds = [
                SWAP_POOL_FEES,
                market_state.key().as_ref(),
                mint_base.key().as_ref()
              ],
              bump,
              token::mint = mint_base,
              token::authority = pool_authority,
              payer = payer,
    )]
    pool_fee_a: Box<Account<'info, TokenAccount>>,

    #[account(init,
        seeds = [
          SWAP_POOL_FEES,
          market_state.key().as_ref(),
          mint_quote.key().as_ref()
        ],
        bump,
        token::mint = mint_quote,
        token::authority = pool_authority,
        payer = payer,
    )]
    pool_fee_b: Box<Account<'info, TokenAccount>>,

    swap_program: AccountInfo<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

impl<'info> OpenBookMarketCreate<'info> {
    fn request_token_a(&self) -> Result<()> {
        crate::token_request(
            Context::new(
                &crate::ID,
                &mut TokenRequest {
                    requester: self.payer.clone(),
                    mint: (*self.mint_base).clone(),
                    info: (*self.info_base).clone(),
                    destination: (*self.pool_token_a).clone(),
                    token_program: self.token_program.clone(),
                },
                &[],
                BTreeMap::new(),
            ),
            1000,
        )
    }

    fn request_token_b(&self) -> Result<()> {
        crate::token_request(
            Context::new(
                &crate::ID,
                &mut TokenRequest {
                    requester: self.payer.clone(),
                    mint: (*self.mint_quote).clone(),
                    info: (*self.info_quote).clone(),
                    destination: (*self.pool_token_b).clone(),
                    token_program: self.token_program.clone(),
                },
                &[],
                BTreeMap::new(),
            ),
            1000,
        )
    }
}

pub fn openbook_market_create_handler(
    ctx: Context<OpenBookMarketCreate>,
    params: OpenBookMarketCreateParams,
) -> Result<()> {
    ctx.accounts.request_token_a()?;
    ctx.accounts.request_token_b()?;

    let market_info = &mut ctx.accounts.market_info;
    market_info.market_state = ctx.accounts.market_state.key();
    market_info.liquidity_level = params.liquidity_level;
    market_info.price_threshold = params.price_threshold;

    let bump = *ctx.bumps.get("pool_state").unwrap();

    let mint_a_key = ctx.accounts.mint_base.key();
    let mint_b_key = ctx.accounts.mint_quote.key();

    let pool_signer_seeds = [
        SWAP_POOL_STATE,
        mint_a_key.as_ref(),
        mint_b_key.as_ref(),
        &[bump],
    ];
    let seeds = [&pool_signer_seeds[..]];

    let swap_context = CpiContext::new_with_signer(
        ctx.accounts.swap_program.to_account_info(),
        dex::InitializeMarket {
            market: ctx.accounts.market_state.to_account_info(),
            coin_mint: ctx.accounts.mint_base.to_account_info(),
            pc_mint: ctx.accounts.mint_quote.to_account_info(),
            coin_vault: InitToken {
                reserve: ctx.accounts.pool_token_a.to_account_info(),
                fees: ctx.accounts.pool_fee_a.to_account_info(),
                mint: ctx.accounts.mint_base.to_account_info(),
            },
            pc_vault: InitToken {
                reserve: ctx.accounts.pool_token_b.to_account_info(),
                fees: ctx.accounts.pool_fee_b.to_account_info(),
                mint: ctx.accounts.mint_quote.to_account_info(),
            },
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
        0, // TODO nonce
        params.quote_dust_threshold,
    )?;

    Ok(())
}
