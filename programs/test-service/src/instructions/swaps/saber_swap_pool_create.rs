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
use anchor_spl::token::{Mint, Token, TokenAccount};
use saber_stable_swap::InitToken;

use crate::instructions::TokenRequest;
use crate::seeds::{
    SWAP_POOL_FEES, SWAP_POOL_INFO, SWAP_POOL_MINT, SWAP_POOL_STATE, SWAP_POOL_TOKENS,
};
use crate::state::{SaberSwapInfo, TokenInfo};

#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone, Eq, PartialEq)]
pub struct SaberSwapPoolCreateParams {
    pub nonce: u8,
    pub liquidity_level: u8,
    pub price_threshold: u16,
}

#[derive(Accounts)]
pub struct SaberSwapPoolCreate<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    #[account(mut)]
    mint_a: Box<Account<'info, Mint>>,

    #[account(mut)]
    mint_b: Box<Account<'info, Mint>>,

    #[account(constraint = info_a.mint == mint_a.key())]
    info_a: Box<Account<'info, TokenInfo>>,

    #[account(constraint = info_b.mint == mint_b.key())]
    info_b: Box<Account<'info, TokenInfo>>,

    #[account(init,
              seeds = [
                SWAP_POOL_INFO,
                mint_a.key().as_ref(),
                mint_b.key().as_ref(),
              ],
              bump,
              space = 8 + std::mem::size_of::<SaberSwapInfo>(),
              payer = payer
    )]
    pool_info: Box<Account<'info, SaberSwapInfo>>,

    #[account(init,
              seeds = [
                SWAP_POOL_STATE,
                mint_a.key().as_ref(),
                mint_b.key().as_ref(),
              ],
              bump,
              space = saber_stable_client::state::SwapInfo::LEN,
              owner = saber_stable_swap::ID,
              payer = payer
    )]
    pool_state: AccountInfo<'info>,

    #[account(seeds = [pool_state.key().as_ref()],
              bump,
              seeds::program = saber_stable_swap::ID
    )]
    pool_authority: AccountInfo<'info>,

    #[account(init,
              seeds = [
                SWAP_POOL_MINT,
                pool_state.key().as_ref()
              ],
              bump,
              mint::decimals = mint_a.decimals,
              mint::authority = pool_authority,
              payer = payer
    )]
    pool_mint: Box<Account<'info, Mint>>,

    #[account(init,
              seeds = [
                SWAP_POOL_TOKENS,
                pool_state.key().as_ref(),
                mint_a.key().as_ref()
              ],
              bump,
              token::mint = mint_a,
              token::authority = pool_authority,
              payer = payer,
    )]
    pool_token_a: Box<Account<'info, TokenAccount>>,

    #[account(init,
              seeds = [
                SWAP_POOL_TOKENS,
                pool_state.key().as_ref(),
                mint_b.key().as_ref()
              ],
              bump,
              token::mint = mint_b,
              token::authority = pool_authority,
              payer = payer,
    )]
    pool_token_b: Box<Account<'info, TokenAccount>>,

    #[account(init,
              seeds = [
                SWAP_POOL_FEES,
                pool_state.key().as_ref(),
                mint_a.key().as_ref()
              ],
              bump,
              token::mint = mint_a,
              token::authority = pool_authority,
              payer = payer,
    )]
    pool_fee_a: Box<Account<'info, TokenAccount>>,

    #[account(init,
        seeds = [
          SWAP_POOL_FEES,
          pool_state.key().as_ref(),
          mint_b.key().as_ref()
        ],
        bump,
        token::mint = mint_b,
        token::authority = pool_authority,
        payer = payer,
    )]
    pool_fee_b: Box<Account<'info, TokenAccount>>,

    #[account(init,
        seeds = [
          SWAP_POOL_FEES,
          pool_state.key().as_ref(),
          pool_mint.key().as_ref()
        ],
        bump,
        token::mint = pool_mint,
        // The LP token authority must be the same as the scratch accounts
        // else withdrawals from the pool do not work as only 1 authority is provided
        token::authority = payer,
        payer = payer,
    )]
    lp_token: Box<Account<'info, TokenAccount>>,

    swap_program: AccountInfo<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

impl<'info> SaberSwapPoolCreate<'info> {
    fn request_token_a(&self) -> Result<()> {
        crate::token_request(
            Context::new(
                &crate::ID,
                &mut TokenRequest {
                    requester: self.payer.clone(),
                    mint: (*self.mint_a).clone(),
                    info: (*self.info_a).clone(),
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
                    mint: (*self.mint_b).clone(),
                    info: (*self.info_b).clone(),
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

pub fn saber_swap_pool_create_handler(
    ctx: Context<SaberSwapPoolCreate>,
    params: SaberSwapPoolCreateParams,
) -> Result<()> {
    ctx.accounts.request_token_a()?;
    ctx.accounts.request_token_b()?;

    let pool_info = &mut ctx.accounts.pool_info;
    pool_info.pool_state = ctx.accounts.pool_state.key();
    pool_info.liquidity_level = params.liquidity_level;
    pool_info.price_threshold = params.price_threshold;

    let bump = *ctx.bumps.get("pool_state").unwrap();

    let mint_a_key = ctx.accounts.mint_a.key();
    let mint_b_key = ctx.accounts.mint_b.key();

    let pool_signer_seeds = [
        SWAP_POOL_STATE,
        mint_a_key.as_ref(),
        mint_b_key.as_ref(),
        &[bump],
    ];
    let seeds = [&pool_signer_seeds[..]];

    let swap_context = CpiContext::new_with_signer(
        ctx.accounts.swap_program.to_account_info(),
        saber_stable_swap::Initialize {
            swap: ctx.accounts.pool_state.to_account_info(),
            swap_authority: ctx.accounts.pool_authority.to_account_info(),
            admin: ctx.accounts.payer.to_account_info(),
            token_a: InitToken {
                reserve: ctx.accounts.pool_token_a.to_account_info(),
                fees: ctx.accounts.pool_fee_a.to_account_info(),
                mint: ctx.accounts.mint_a.to_account_info(),
            },
            token_b: InitToken {
                reserve: ctx.accounts.pool_token_b.to_account_info(),
                fees: ctx.accounts.pool_fee_b.to_account_info(),
                mint: ctx.accounts.mint_b.to_account_info(),
            },
            pool_mint: ctx.accounts.pool_mint.to_account_info(),
            output_lp: ctx.accounts.lp_token.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
        &seeds,
    );

    saber_stable_swap::initialize(
        swap_context,
        bump,
        100,
        saber_stable_client::fees::Fees {
            admin_trade_fee_numerator: 1,
            admin_trade_fee_denominator: 400,
            admin_withdraw_fee_numerator: 1,
            admin_withdraw_fee_denominator: 200,
            trade_fee_numerator: 1,
            trade_fee_denominator: 100,
            withdraw_fee_numerator: 1,
            withdraw_fee_denominator: 100,
        },
    )?;

    Ok(())
}
