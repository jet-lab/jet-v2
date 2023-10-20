// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright (C) 2023 JET PROTOCOL HOLDINGS, LLC.
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

use anchor_spl::token::{Mint, Token};
use jet_airspace::state::Airspace;
use jet_margin::{AdapterConfig, TokenConfig};
use orca_whirlpool::program::Whirlpool;

use crate::*;

#[derive(Accounts)]
pub struct CreateWhirlpoolConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The authority that must sign to make this change
    pub authority: Signer<'info>,

    /// The airspace being modified, disable testing
    #[account(has_one = authority @ MarginOrcaErrorCode::WrongAirspaceAuthorization)]
    pub airspace: Box<Account<'info, Airspace>>,

    /// Check that there is an adapter config for this program in margin
    #[account(
        has_one = airspace @ MarginOrcaErrorCode::WrongAirspaceAuthorization,
        constraint = adapter_config.adapter_program == crate::ID
    )]
    pub adapter_config: Box<Account<'info, AdapterConfig>>,

    #[account(init,
                seeds = [
                    seeds::ORCA_ADAPTER_CONFIG,
                    airspace.key().as_ref(),
                    mint_a.key().as_ref(),
                    mint_b.key().as_ref(),
                ],
                bump,
                payer = payer,
                space = WhirlpoolConfig::SIZE
    )]
    pub whirlpool_config: Box<Account<'info, WhirlpoolConfig>>,

    #[account(constraint = token_a_config.mint == mint_a.key())]
    pub token_a_config: Box<Account<'info, TokenConfig>>,
    #[account(constraint = token_b_config.mint == mint_b.key())]
    pub token_b_config: Box<Account<'info, TokenConfig>>,

    pub mint_a: Box<Account<'info, Mint>>,
    pub mint_b: Box<Account<'info, Mint>>,

    /// Mints tokens representing the amount of liquidity of positions in the margin account
    #[account(init,
        seeds = [
            seeds::POSITION_NOTES,
            whirlpool_config.key().as_ref(),
        ],
        bump,
        payer = payer,
        mint::decimals = 0,
        mint::authority = whirlpool_config,
        mint::freeze_authority = whirlpool_config,
    )]
    pub margin_position_mint: Box<Account<'info, Mint>>,

    /// The address of the Orca program
    pub orca_program: Program<'info, Whirlpool>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn create_whirlpool_config_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CreateWhirlpoolConfig<'info>>,
) -> Result<()> {
    let whirlpool_config = &mut ctx.accounts.whirlpool_config;
    whirlpool_config.airspace = ctx.accounts.airspace.key();
    whirlpool_config.mint_a = ctx.accounts.token_a_config.underlying_mint;
    whirlpool_config.mint_b = ctx.accounts.token_b_config.underlying_mint;
    whirlpool_config.mint_a_decimals = ctx.accounts.mint_a.decimals;
    whirlpool_config.mint_b_decimals = ctx.accounts.mint_b.decimals;
    match ctx.accounts.token_a_config.oracle().unwrap() {
        jet_margin::TokenOracle::Pyth { price, .. } => {
            whirlpool_config.token_a_oracle = price;
        }
    }
    match ctx.accounts.token_b_config.oracle().unwrap() {
        jet_margin::TokenOracle::Pyth { price, .. } => {
            whirlpool_config.token_b_oracle = price;
        }
    }
    whirlpool_config.position_mint = ctx.accounts.margin_position_mint.key();
    whirlpool_config.bump = [*ctx.bumps.get("whirlpool_config").unwrap()];

    // TODO: emit an event

    Ok(())
}
