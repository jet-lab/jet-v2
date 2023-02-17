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
use anchor_spl::token::{Mint, Token, TokenAccount};

use jet_program_common::Number128;
use saber_stable_swap::{SwapOutput, SwapToken, SwapUserContext};

use crate::seeds::{SWAP_POOL_FEES, SWAP_POOL_INFO, SWAP_POOL_MINT, SWAP_POOL_TOKENS, TOKEN_INFO};
use crate::state::{SaberSwapInfo, TokenInfo};

#[derive(Accounts)]
pub struct SaberSwapPoolBalance<'info> {
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
    pool_info: Box<Account<'info, SaberSwapInfo>>,

    #[account(mut)]
    pool_state: AccountInfo<'info>,

    #[account(seeds = [pool_state.key().as_ref()],
              bump,
              seeds::program = saber_program
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
                SWAP_POOL_FEES,
                pool_state.key().as_ref(),
                mint_a.key().as_ref()
              ],
              bump,
    )]
    pool_fee_a: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        seeds = [
          SWAP_POOL_FEES,
          pool_state.key().as_ref(),
          mint_b.key().as_ref()
        ],
        bump,
    )]
    pool_fee_b: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        seeds = [
          SWAP_POOL_FEES,
          pool_state.key().as_ref(),
          pool_mint.key().as_ref()
        ],
        bump,
    )]
    lp_token: Box<Account<'info, TokenAccount>>,

    saber_program: AccountInfo<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

pub fn saber_swap_pool_balance_handler(ctx: Context<SaberSwapPoolBalance>) -> Result<()> {
    let SaberSwapPoolBalance {
        mint_a,
        mint_b,
        pool_info,
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

    apply_changes(&ctx, (change_a, price_a), (change_b, price_b))?;

    Ok(())
}

fn apply_changes(
    ctx: &Context<SaberSwapPoolBalance>,
    values_a: (Number128, Number128),
    values_b: (Number128, Number128),
) -> Result<()> {
    let (change_a, price_a) = values_a;
    let (change_b, price_b) = values_b;
    let mint_a_key = ctx.accounts.mint_a.key();
    let mint_b_key = ctx.accounts.mint_b.key();
    let pool_info_signer_seeds = [
        SWAP_POOL_INFO,
        mint_a_key.as_ref(),
        mint_b_key.as_ref(),
        &[*ctx.bumps.get("pool_info").unwrap()],
    ];

    let token_mint_a_signer_seeds = [
        TOKEN_INFO,
        mint_a_key.as_ref(),
        &[ctx.accounts.info_a.bump_seed],
    ];
    let token_mint_b_signer_seeds = [
        TOKEN_INFO,
        mint_b_key.as_ref(),
        &[ctx.accounts.info_b.bump_seed],
    ];

    let (deposit_a, withdraw_a) =
        calculate_token_change(change_a, price_a, ctx.accounts.mint_a.decimals);
    let (deposit_b, withdraw_b) =
        calculate_token_change(change_b, price_b, ctx.accounts.mint_b.decimals);

    if (withdraw_a + withdraw_b) > 0 {
        let withdraw_ctx = CpiContext::new(
            ctx.accounts.saber_program.to_account_info(),
            saber_stable_swap::Withdraw {
                user: SwapUserContext {
                    token_program: ctx.accounts.token_program.to_account_info(),
                    swap_authority: ctx.accounts.pool_authority.to_account_info(),
                    user_authority: ctx.accounts.payer.to_account_info(),
                    swap: ctx.accounts.pool_state.to_account_info(),
                },
                pool_mint: ctx.accounts.pool_mint.to_account_info(),
                input_lp: ctx.accounts.lp_token.to_account_info(),
                output_a: SwapOutput {
                    user_token: SwapToken {
                        user: ctx.accounts.scratch_a.to_account_info(),
                        reserve: ctx.accounts.pool_token_a.to_account_info(),
                    },
                    fees: ctx.accounts.pool_fee_a.to_account_info(),
                },
                output_b: SwapOutput {
                    user_token: SwapToken {
                        user: ctx.accounts.scratch_b.to_account_info(),
                        reserve: ctx.accounts.pool_token_b.to_account_info(),
                    },
                    fees: ctx.accounts.pool_fee_b.to_account_info(),
                },
            },
        );

        saber_stable_swap::withdraw(
            withdraw_ctx.with_signer(&[&pool_info_signer_seeds]),
            // We implicitly rely on the LP tokens being ~ vaults + fees,
            // otherwise we'd have to calculate the precise LP tokens to withdraw.
            // As the pool isn't really impacted by oracle changes as it's stable, this is fine.
            withdraw_a + withdraw_b,
            0,
            0,
        )?;

        let scratch_a_remaining =
            anchor_spl::token::accessor::amount(&ctx.accounts.scratch_a.to_account_info())?;
        anchor_spl::token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Burn {
                    mint: ctx.accounts.mint_a.to_account_info(),
                    from: ctx.accounts.scratch_a.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            scratch_a_remaining,
        )?;
        let scratch_b_remaining =
            anchor_spl::token::accessor::amount(&ctx.accounts.scratch_b.to_account_info())?;
        anchor_spl::token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Burn {
                    mint: ctx.accounts.mint_b.to_account_info(),
                    from: ctx.accounts.scratch_b.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            scratch_b_remaining,
        )?;
    } else if (deposit_a + deposit_b) > 0 {
        anchor_spl::token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: ctx.accounts.mint_a.to_account_info(),
                    to: ctx.accounts.scratch_a.to_account_info(),
                    authority: ctx.accounts.info_a.to_account_info(),
                },
            )
            .with_signer(&[&token_mint_a_signer_seeds]),
            deposit_a,
        )?;
        anchor_spl::token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: ctx.accounts.mint_b.to_account_info(),
                    to: ctx.accounts.scratch_b.to_account_info(),
                    authority: ctx.accounts.info_b.to_account_info(),
                },
            )
            .with_signer(&[&token_mint_b_signer_seeds]),
            deposit_b,
        )?;

        let deposit_ctx = CpiContext::new(
            ctx.accounts.saber_program.to_account_info(),
            saber_stable_swap::Deposit {
                user: SwapUserContext {
                    token_program: ctx.accounts.token_program.to_account_info(),
                    swap_authority: ctx.accounts.pool_authority.to_account_info(),
                    user_authority: ctx.accounts.payer.to_account_info(),
                    swap: ctx.accounts.pool_state.to_account_info(),
                },
                input_a: SwapToken {
                    user: ctx.accounts.scratch_a.to_account_info(),
                    reserve: ctx.accounts.pool_token_a.to_account_info(),
                },
                input_b: SwapToken {
                    user: ctx.accounts.scratch_b.to_account_info(),
                    reserve: ctx.accounts.pool_token_b.to_account_info(),
                },
                pool_mint: ctx.accounts.pool_mint.to_account_info(),
                output_lp: ctx.accounts.lp_token.to_account_info(),
            },
        );

        saber_stable_swap::deposit(
            deposit_ctx.with_signer(&[&pool_info_signer_seeds]),
            deposit_a,
            deposit_b,
            1,
        )?;
    }

    Ok(())
}

fn read_price(pyth_price: &AccountInfo) -> Number128 {
    let price_result = pyth_sdk_solana::load_price_feed_from_account_info(pyth_price).unwrap();
    let price_value = price_result.get_price_unchecked();

    Number128::from_decimal(price_value.price, price_value.expo)
}

/// Calculate the number of tokens to deposit or withdraw for a side of the pool.
/// The tokens returned are expressed as (deposit, withdraw).
fn calculate_token_change(change: Number128, price: Number128, decimals: u8) -> (u64, u64) {
    match change {
        a if a > Number128::ZERO => {
            let tokens = (change / price).as_u64(-(decimals as i32));
            if tokens == 0 {
                (0, 0)
            } else {
                (0, tokens)
            }
        }
        a if a == Number128::ZERO => (0, 0),
        _ => {
            let tokens =
                ((change * Number128::from_decimal(-1, 0)) / price).as_u64(-(decimals as i32));
            if tokens == 0 {
                (0, 0)
            } else {
                (tokens, 0)
            }
        }
    }
}
