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
use anchor_lang::solana_program::program::invoke;
use anchor_spl::token;

use jet_margin::MarginAccount;
use jet_margin_pool::{
    cpi::accounts::{Deposit, Withdraw},
    program::JetMarginPool,
    ChangeKind,
};

declare_id!("JPMAa5dnWLFRvUsumawFcGhnwikqZziLLfqn9SLNXPN");

mod instructions;
use instructions::*;

/// The maximum swap split percentage
pub const ROUTE_SWAP_MAX_SPLIT: u8 = 90;
/// The minimum swap split percentage
pub const ROUTE_SWAP_MIN_SPLIT: u8 = 100 - ROUTE_SWAP_MAX_SPLIT;

#[program]
mod jet_margin_swap {
    use super::*;

    pub fn margin_swap(
        ctx: Context<MarginSplSwap>,
        withdrawal_change_kind: ChangeKind,
        withdrawal_amount: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        margin_spl_swap_handler(
            ctx,
            withdrawal_change_kind,
            withdrawal_amount,
            minimum_amount_out,
        )
    }

    /// Swap using Orca Whirlpool for stable pools
    pub fn orca_whirlpool_swap(
        ctx: Context<OrcaWhirlpoolSwap>,
        withdrawal_change_kind: ChangeKind,
        withdrawal_amount: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        orca_whirlpool_swap_handler(
            ctx,
            withdrawal_change_kind,
            withdrawal_amount,
            minimum_amount_out,
        )
    }

    /// Swap using Saber for stable pools
    pub fn saber_stable_swap(ctx: Context<SaberSwapInfo>) -> Result<()> {
        saber_stable_swap_handler(ctx)
    }

    /// Route a swap to one or more venues
    pub fn route_swap<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, RouteSwap<'info>>,
        withdrawal_change_kind: ChangeKind,
        withdrawal_amount: u64,
        minimum_amount_out: u64,
        swap_routes: [SwapRouteDetail; 3],
    ) -> Result<()> {
        route_swap_handler(
            ctx,
            withdrawal_change_kind,
            withdrawal_amount,
            minimum_amount_out,
            swap_routes,
        )
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
pub enum ErrorCode {
    #[msg("Zero tokens have been withdrawn from a pool for the swap")]
    NoSwapTokensWithdrawn,

    #[msg("An invalid swap route has been provided")]
    InvalidSwapRoute,

    #[msg("An invalid swap route parameter has been provided")]
    InvalidSwapRouteParam,

    #[msg("The swap exceeds the maximum slippage tolerance")]
    SlippageExceeded,

    #[msg("The instruction should not be called directly, use route_swap")]
    DisallowedDirectInstruction,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub enum SwapRouteIdentifier {
    Empty = 0,
    Spl,
    OrcaWhirlpool,
    SaberStable,
}

impl Default for SwapRouteIdentifier {
    fn default() -> Self {
        Self::Empty
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub struct SwapRouteDetail {
    pub route_a: SwapRouteIdentifier,
    pub route_b: SwapRouteIdentifier,
    pub destination_mint: Pubkey,
    pub split: u8,
}

impl Default for SwapRouteDetail {
    fn default() -> Self {
        Self {
            route_a: SwapRouteIdentifier::Empty,
            route_b: SwapRouteIdentifier::Empty,
            destination_mint: Default::default(),
            split: 0,
        }
    }
}

impl SwapRouteDetail {
    pub fn validate(&self) -> Result<bool> {
        use SwapRouteIdentifier::*;
        // There's an anchor bug that gets triggered when using these consts
        // const MIN: u8 = ROUTE_SWAP_MIN_SPLIT - 1;
        // const MAX: u8 = ROUTE_SWAP_MAX_SPLIT + 1;
        match (self.route_a, self.route_b, self.split) {
            (Empty, Empty, _) => Ok(false),
            (_, Empty, 0) => Ok(true),
            // We limit splits to 95%, thus 96+ or 4- are not allowed
            (Empty, _, _) | (_, _, 0..=9) | (_, _, 91..) => {
                Err(error!(ErrorCode::InvalidSwapRouteParam))
            }
            _ => Ok(true),
        }
    }
}