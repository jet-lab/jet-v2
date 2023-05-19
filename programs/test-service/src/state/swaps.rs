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

/// Information about a SPL swap pool
#[account]
pub struct SplSwapInfo {
    /// The address of the pool state
    pub pool_state: Pubkey,

    /// The magnitude of liquidity to provide the pool
    pub liquidity_level: u8,

    /// The allowance of price deviation before the pool may be rebalanced
    pub price_threshold: u16,
}

/// Information about a Saber swap pool
#[account]
pub struct SaberSwapInfo {
    /// The address of the pool state
    pub pool_state: Pubkey,

    /// The magnitude of liquidity to provide the pool
    pub liquidity_level: u8,

    /// The allowance of price deviation before the pool may be rebalanced
    pub price_threshold: u16,
}

/// Information about an OpebBook market
#[account]
pub struct OpenBookMarketInfo {
    /// The address of the market state
    pub market_state: Pubkey,

    /// The initial spread from the market price to quote at
    pub initial_spread: u16,

    /// The incremental spread to quote at after the starting spread
    pub incremental_spread: u16,

    /// The multiplier to apply when quoting at each level.
    /// 8 bids and asks are placed at each time.
    pub basket_sizes: [u8; 8],

    /// The amount in USD to provide liquidity per basket.
    /// If the sum of baskets is 10 and each unit is 500, then $5000 of liquidity is provided.
    /// This value is per side of the book.
    pub basket_liquidity: u64,
}
