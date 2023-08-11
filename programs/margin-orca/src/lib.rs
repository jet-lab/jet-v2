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

// Allow this until fixed upstream
#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

use jet_margin::MarginAccount;
use orca_whirlpool::state::OpenPositionBumps;

declare_id!("6FfgwPxT7WSWLwPk4U4axXmnCgpehpahMcFMVoQgZ2dA");

mod instructions;
use instructions::*;
mod state;
pub use state::*;

pub mod seeds {
    use super::constant;

    #[constant]
    pub const ORCA_ADAPTER_CONFIG: &[u8] = b"orca_adapter_config";

    #[constant]
    pub const POSITION_NOTES: &[u8] = b"orca_position_notes";

    #[constant]
    pub const POSITION_METADATA: &[u8] = b"orca_position_metadata";

    #[constant]
    pub const POSITION_MINT: &[u8] = b"orca_position_mint";
}

/// The fee charged for liquidation swaps (bps)
pub const LIQUIDATION_FEE: u64 = 3_00;

#[program]
mod jet_margin_orca {
    use super::*;

    pub fn create_whirlpool_config<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateWhirlpoolConfig<'info>>,
    ) -> Result<()> {
        create_whirlpool_config_handler(ctx)
    }

    // Collect reward: https://explorer.solana.com/tx/2gQd6mPTZfaKLQiexPQf2bEobkupqii9xReoYRJbVdeYqiXiUtdkt4WEQ39UM2QrgUeZhDX5bn7JbVJWx7ncEgrT?cluster=mainnet-beta
    // Close position
    // Close bundled position?
    // Decrease liquidity
    // Open position: https://explorer.solana.com/tx/5dUnLAdzYYgeTbqw5H2ieVpdEFrVWxXFwKPiX7UY3Wk7nJvm7yqEg4rYfEhSfqT5RwSMndYrKGMzvkg4HmvdEDE2?cluster=mainnet-beta
    // Increate liquidity: https://explorer.solana.com/tx/5HZGVjnRKohd4uXiT1h14WSNztVgMs5iaRnkmDiM8dzRmM9WajbMMiuLLik447vhwbYNyvuyxNzZtaBoqjXKUFeS?cluster=mainnet-beta

    pub fn open_whirlpool_position<'info>(
        ctx: Context<'_, '_, '_, 'info, OpenWhirlpoolPosition<'info>>,
        bumps: OpenPositionBumps,
        seed: u64,
        tick_lower_index: i32,
        tick_upper_index: i32,
    ) -> Result<()> {
        open_whirlpool_position_handler(ctx, bumps, seed, tick_lower_index, tick_upper_index)
    }

    pub fn close_whirlpool_position<'info>(
        ctx: Context<'_, '_, '_, 'info, CloseWhirlpoolPosition<'info>>,
    ) -> Result<()> {
        close_whirlpool_position_handler(ctx)
    }

    pub fn increase_liquidity<'info>(
        ctx: Context<'_, '_, '_, 'info, ModifyLiquidity<'info>>,
        liquidity_amount: u128,
        token_max_a: u64,
        token_max_b: u64,
    ) -> Result<()> {
        modify_liquidity_handler(ctx, true, liquidity_amount, token_max_a, token_max_b)
    }

    pub fn decrease_liquidity<'info>(
        ctx: Context<'_, '_, '_, 'info, ModifyLiquidity<'info>>,
        liquidity_amount: u128,
        token_max_a: u64,
        token_max_b: u64,
    ) -> Result<()> {
        modify_liquidity_handler(ctx, false, liquidity_amount, token_max_a, token_max_b)
    }

    pub fn collect_reward<'info>(
        ctx: Context<'_, '_, '_, 'info, CollectReward<'info>>,
        reward_index: u8,
    ) -> Result<()> {
        collect_reward_handler(ctx, reward_index)
    }

    pub fn margin_refresh_position<'info>(
        ctx: Context<'_, '_, '_, 'info, MarginRefreshPosition<'info>>,
    ) -> Result<()> {
        margin_refresh_position_handler(ctx)
    }

    pub fn register_margin_position<'info>(
        ctx: Context<'_, '_, '_, 'info, RegisterMarginPosition<'info>>,
    ) -> Result<()> {
        register_margin_position_handler(ctx)
    }

    pub fn close_margin_position<'info>(
        ctx: Context<'_, '_, '_, 'info, CloseMarginPosition<'info>>,
    ) -> Result<()> {
        close_margin_position_handler(ctx)
    }
}

#[derive(Accounts)]
pub struct MarginPoolInfo<'info> {
    /// CHECK:
    #[account(mut)]
    pub margin_pool: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub deposit_note_mint: UncheckedAccount<'info>,
}

#[error_code]
pub enum MarginOrcaErrorCode {
    #[msg("Wrong airspace authorization")]
    WrongAirspaceAuthorization = 11_000,

    #[msg("There are no empty position slots for this whirlpool")]
    PositionsFull,

    #[msg("An internal error occurred while updating position")]
    PositionUpdateError,

    #[msg("A supplied argument is invalid")]
    InvalidArgument,

    #[msg("Encountered an unexpected arithmetic error")]
    ArithmeticError,

    #[msg("Oracle is invalid")]
    InvalidOracle,

    #[msg("The account is not empty")]
    AccountNotEmpty,
}

#[event]
pub struct EmptyEvent {}
