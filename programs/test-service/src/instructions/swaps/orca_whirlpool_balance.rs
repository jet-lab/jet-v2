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

use anchor_lang::solana_program::program::invoke;
use anchor_lang::{prelude::*, solana_program::program::invoke_signed};
use anchor_spl::token::{Mint, Token, TokenAccount};

use jet_program_common::Number128;

use crate::seeds::{SWAP_POOL_INFO, SWAP_POOL_MINT, SWAP_POOL_TOKENS, TOKEN_INFO};
use crate::state::{SplSwapInfo, TokenInfo};

// todo - fixme
#[derive(Accounts)]
pub struct WhirlpoolBalance<'info> {
    payer: Signer<'info>,

    #[account(mut, token::authority = payer)]
    scratch_a: Box<Account<'info, TokenAccount>>,

    #[account(mut, token::authority = payer)]
    scratch_b: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    mint_a: Box<Account<'info, Mint>>,

    #[account(mut)]
    mint_b: Box<Account<'info, Mint>>,

    #[account(constraint = info_a.mint == mint_a.key(),
              constraint = info_a.pyth_price == pyth_price_a.key()
    )]
    info_a: Box<Account<'info, TokenInfo>>,

    #[account(constraint = info_b.mint == mint_b.key(),
              constraint = info_b.pyth_price == pyth_price_b.key()
    )]
    info_b: Box<Account<'info, TokenInfo>>,

    pyth_price_a: AccountInfo<'info>,
    pyth_price_b: AccountInfo<'info>,

    #[account(has_one = pool_state,
              seeds = [
                SWAP_POOL_INFO,
                mint_a.key().as_ref(),
                mint_b.key().as_ref()
              ],
              bump,
    )]
    pool_info: Box<Account<'info, SplSwapInfo>>,

    #[account(mut)]
    pool_state: AccountInfo<'info>,

    #[account(seeds = [pool_state.key().as_ref()],
              bump,
              seeds::program = spl_token_swap::ID
    )]
    pool_authority: AccountInfo<'info>,

    #[account(mut,
              seeds = [
                SWAP_POOL_MINT,
                pool_state.key().as_ref()
              ],
              bump,
    )]
    pool_mint: Box<Account<'info, Mint>>,

    #[account(mut,
              seeds = [
                SWAP_POOL_TOKENS,
                pool_state.key().as_ref(),
                mint_a.key().as_ref()
              ],
              bump,
    )]
    pool_token_a: Box<Account<'info, TokenAccount>>,

    #[account(mut,
              seeds = [
                SWAP_POOL_TOKENS,
                pool_state.key().as_ref(),
                mint_b.key().as_ref()
              ],
              bump,
    )]
    pool_token_b: Box<Account<'info, TokenAccount>>,

    #[account(mut,
              seeds = [
                SWAP_POOL_TOKENS,
                pool_state.key().as_ref(),
                pool_mint.key().as_ref()
              ],
              bump,
    )]
    pool_fees: Box<Account<'info, TokenAccount>>,

    swap_program: AccountInfo<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

pub fn whirlpool_balance_handler(ctx: Context<WhirlpoolBalance>) -> Result<()> {
    let WhirlpoolBalance {
        mint_a,
        mint_b,
        scratch_a,
        scratch_b,
        pool_info,
        info_a,
        info_b,
        pyth_price_a,
        pyth_price_b,
        pool_token_a,
        pool_token_b,
        ..
    } = &ctx.accounts;

    let current_amount_a = Number128::from_decimal(pool_token_a.amount, -(mint_a.decimals as i32));
    let current_amount_b = Number128::from_decimal(pool_token_b.amount, -(mint_b.decimals as i32));

    let price_a = read_price(pyth_price_a);
    let price_b = read_price(pyth_price_b);

    let value_a = price_a * current_amount_a;
    let value_b = price_b * current_amount_b;

    let liquidity_factor = Number128::from_decimal(1, pool_info.liquidity_level);

    let change_a = value_a - liquidity_factor;
    let change_b = value_b - liquidity_factor;

    apply_change(&ctx, change_a, price_a, scratch_a, mint_a, info_a)?;
    apply_change(&ctx, change_b, price_b, scratch_b, mint_b, info_b)?;

    Ok(())
}

fn apply_change<'info>(
    ctx: &Context<WhirlpoolBalance<'info>>,
    change: Number128,
    price: Number128,
    scratch: &Account<'info, TokenAccount>,
    mint: &Account<'info, Mint>,
    info: &Account<'info, TokenInfo>,
) -> Result<()> {
    let mint_a_key = ctx.accounts.mint_a.key();
    let mint_b_key = ctx.accounts.mint_b.key();
    let pool_info_signer_seeds = [
        SWAP_POOL_INFO,
        mint_a_key.as_ref(),
        mint_b_key.as_ref(),
        &[*ctx.bumps.get("pool_info").unwrap()],
    ];

    let mint_key = mint.key();
    let token_mint_signer_seeds = [TOKEN_INFO, mint_key.as_ref(), &[info.bump_seed]];

    if change > Number128::ZERO {
        let tokens = (change / price).as_u64(-(mint.decimals as i32));

        if tokens == 0 {
            return Ok(());
        }

        let ix = spl_token_swap::instruction::withdraw_single_token_type_exact_amount_out(
            ctx.accounts.swap_program.key,
            ctx.accounts.token_program.key,
            ctx.accounts.pool_state.key,
            ctx.accounts.pool_authority.key,
            &ctx.accounts.pool_info.key(),
            &ctx.accounts.pool_mint.key(),
            &ctx.accounts.pool_fees.key(),
            &ctx.accounts.pool_fees.key(),
            &ctx.accounts.pool_token_a.key(),
            &ctx.accounts.pool_token_b.key(),
            &scratch.key(),
            spl_token_swap::instruction::WithdrawSingleTokenTypeExactAmountOut {
                destination_token_amount: tokens,
                maximum_pool_token_amount: u64::MAX,
            },
        )?;

        invoke_signed(
            &ix,
            &[
                ctx.accounts.pool_state.to_account_info(),
                ctx.accounts.pool_authority.to_account_info(),
                ctx.accounts.pool_info.to_account_info(),
                ctx.accounts.pool_mint.to_account_info(),
                ctx.accounts.pool_fees.to_account_info(),
                ctx.accounts.pool_token_a.to_account_info(),
                ctx.accounts.pool_token_b.to_account_info(),
                scratch.to_account_info(),
                ctx.accounts.pool_fees.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
            &[&pool_info_signer_seeds],
        )?;

        let scratch_remaining = anchor_spl::token::accessor::amount(&scratch.to_account_info())?;
        anchor_spl::token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Burn {
                    mint: mint.to_account_info(),
                    from: scratch.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            scratch_remaining,
        )?;
    } else {
        let tokens =
            ((change * Number128::from_decimal(-1, 0)) / price).as_u64(-(mint.decimals as i32));

        if tokens == 0 {
            return Ok(());
        }

        anchor_spl::token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: mint.to_account_info(),
                    to: scratch.to_account_info(),
                    authority: info.to_account_info(),
                },
            )
            .with_signer(&[&token_mint_signer_seeds]),
            tokens,
        )?;

        let ix = spl_token_swap::instruction::deposit_single_token_type_exact_amount_in(
            ctx.accounts.swap_program.key,
            ctx.accounts.token_program.key,
            ctx.accounts.pool_state.key,
            ctx.accounts.pool_authority.key,
            ctx.accounts.payer.key,
            &scratch.key(),
            &ctx.accounts.pool_token_a.key(),
            &ctx.accounts.pool_token_b.key(),
            &ctx.accounts.pool_mint.key(),
            &ctx.accounts.pool_fees.key(),
            spl_token_swap::instruction::DepositSingleTokenTypeExactAmountIn {
                source_token_amount: tokens,
                minimum_pool_token_amount: 0,
            },
        )?;

        invoke(
            &ix,
            &[
                ctx.accounts.pool_state.to_account_info(),
                ctx.accounts.pool_authority.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                scratch.to_account_info(),
                ctx.accounts.pool_token_a.to_account_info(),
                ctx.accounts.pool_token_b.to_account_info(),
                ctx.accounts.pool_mint.to_account_info(),
                ctx.accounts.pool_fees.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
        )?;
    }

    Ok(())
}

fn read_price(pyth_price: &AccountInfo) -> Number128 {
    let price_result = pyth_sdk_solana::load_price_feed_from_account_info(pyth_price).unwrap();
    let price_value = price_result.get_price_unchecked();

    Number128::from_decimal(price_value.price, price_value.expo)
}
