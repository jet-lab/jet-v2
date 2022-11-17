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

use anchor_lang::solana_program::program_pack::Pack;
use anchor_lang::{prelude::*, solana_program::program::invoke_signed};
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::instructions::TokenRequest;
use crate::seeds::{SWAP_POOL_MINT, SWAP_POOL_STATE, SWAP_POOL_TOKENS};
use crate::state::TokenInfo;

#[derive(Accounts)]
pub struct SplSwapPoolCreate<'info> {
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
                SWAP_POOL_STATE,
                mint_a.key().as_ref(),
                mint_b.key().as_ref()
              ],
              bump,
              space = 1 + spl_token_swap::state::SwapV1::LEN,
              owner = spl_token_swap::ID,
              payer = payer
    )]
    pool_state: AccountInfo<'info>,

    #[account(seeds = [pool_state.key().as_ref()],
              bump,
              seeds::program = spl_token_swap::ID
    )]
    pool_authority: AccountInfo<'info>,

    #[account(init,
              seeds = [
                SWAP_POOL_MINT,
                pool_state.key().as_ref()
              ],
              bump,
              mint::decimals = 6,
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
                SWAP_POOL_TOKENS,
                pool_state.key().as_ref(),
                pool_mint.key().as_ref()
              ],
              bump,
              token::mint = pool_mint,
              token::authority = payer,
              payer = payer,
    )]
    pool_fees: Box<Account<'info, TokenAccount>>,

    swap_program: AccountInfo<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

impl<'info> SplSwapPoolCreate<'info> {
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

pub fn spl_swap_pool_create_handler(ctx: Context<SplSwapPoolCreate>) -> Result<()> {
    ctx.accounts.request_token_a()?;
    ctx.accounts.request_token_b()?;

    let bump = *ctx.bumps.get("pool_state").unwrap();

    let ix = spl_token_swap::instruction::initialize(
        ctx.accounts.swap_program.key,
        ctx.accounts.token_program.key,
        ctx.accounts.pool_state.key,
        ctx.accounts.pool_authority.key,
        &ctx.accounts.pool_token_a.key(),
        &ctx.accounts.pool_token_b.key(),
        &ctx.accounts.pool_mint.key(),
        &ctx.accounts.pool_fees.key(),
        &ctx.accounts.pool_fees.key(),
        bump,
        spl_token_swap::curve::fees::Fees {
            // The fee parameters are taken from one of spl-token-swap tests
            trade_fee_numerator: 1,
            trade_fee_denominator: 400,
            owner_trade_fee_numerator: 2,
            owner_trade_fee_denominator: 500,
            owner_withdraw_fee_numerator: 4,
            owner_withdraw_fee_denominator: 100,
            host_fee_numerator: 1,
            host_fee_denominator: 100,
        },
        spl_token_swap::curve::base::SwapCurve {
            curve_type: spl_token_swap::curve::base::CurveType::ConstantProduct,
            calculator: Box::new(spl_token_swap::curve::constant_product::ConstantProductCurve),
        },
    )?;

    let mint_a_key = ctx.accounts.mint_a.key();
    let mint_b_key = ctx.accounts.mint_b.key();

    let pool_signer_seeds = [
        SWAP_POOL_STATE,
        mint_a_key.as_ref(),
        mint_b_key.as_ref(),
        &[bump],
    ];

    invoke_signed(
        &ix,
        &[
            ctx.accounts.pool_state.to_account_info(),
            ctx.accounts.pool_authority.to_account_info(),
            ctx.accounts.pool_token_a.to_account_info(),
            ctx.accounts.pool_token_b.to_account_info(),
            ctx.accounts.pool_mint.to_account_info(),
            ctx.accounts.pool_fees.to_account_info(),
            ctx.accounts.pool_fees.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
        &[&pool_signer_seeds],
    )?;

    Ok(())
}
