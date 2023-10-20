use std::{
    collections::HashSet,
    convert::{TryFrom, TryInto},
};

use anchor_lang::prelude::*;
use jet_margin::{AdapterResult, PositionChange, PriceChangeInfo};
use jet_program_common::{
    traits::{SafeDiv, SafeMul},
    Number128,
};
use orca_whirlpool::{
    manager::liquidity_manager::calculate_liquidity_token_deltas,
    math::sqrt_price_from_tick_index,
    state::{Position as WhirlpoolPosition, Whirlpool},
};

use crate::*;

/// The number of positions in a single pool a user is permitted.
pub const MAX_POSITIONS: usize = 10;

macro_rules! declare_account_size {
    ($name:ident, $size:expr) => {
        impl $name {
            pub const SIZE: usize = $size;
        }

        const _: () = assert!(
            $name::SIZE >= (8 + std::mem::size_of::<$name>()),
            concat!(
                "declared account size is too low compared to actual size: ",
                stringify!($name)
            )
        );
    };
}

#[account]
pub struct WhirlpoolConfig {
    pub airspace: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub position_mint: Pubkey,
    pub token_a_oracle: Pubkey,
    pub token_b_oracle: Pubkey,
    pub mint_a_decimals: u8,
    pub mint_b_decimals: u8,
    pub(crate) bump: [u8; 1],
    pub _reserved: [u8; 5],
    // TODO flags, enabled|paused|withdrawal_only
}

declare_account_size!(WhirlpoolConfig, 208);

impl WhirlpoolConfig {
    pub fn authority_seeds(&self) -> [&[u8]; 5] {
        [
            seeds::ORCA_ADAPTER_CONFIG,
            self.airspace.as_ref(),
            self.mint_a.as_ref(),
            self.mint_b.as_ref(),
            &self.bump,
        ]
    }
}

#[account()]
#[derive(Default)]
pub struct PositionMetadata {
    pub owner: Pubkey,
    pub whirlpool_config: Pubkey,

    // Cached data required to refresh positions
    /// The oracle price for token A, stored as a `Number128` with fixed precision.
    pub price_a: [u8; 16],
    /// The oracle price for token B, stored as a `Number128` with fixed precision.
    pub price_b: [u8; 16],
    pub price_a_timestamp: i64,
    pub price_b_timestamp: i64,

    pub positions: [PositionDetails; MAX_POSITIONS],
    // TODO: what other information would we like to track?
}

declare_account_size!(PositionMetadata, 1480);

#[derive(Clone, Copy, Default, AnchorSerialize, AnchorDeserialize)]
pub struct PositionDetails {
    pub address: Pubkey,
    pub whirlpool: Pubkey,
    pub liquidity: u128,
    /// The current sqrt price from the whirlpool.
    /// RISK: we do not currently use this, perhaps we can use it to compare
    /// against the oracle price.
    pub current_sqrt_price: u128,
    /// The current tick index from the whirlpool
    pub tick_index_current: i32,
    pub tick_index_lower: i32,
    pub tick_index_upper: i32,
    pub last_refresh: i64,
    pub fee_owed_a: u64,
    pub fee_owed_b: u64,
    // TODO: more info, any flags?
}

impl PositionMetadata {
    pub fn free_position(&self) -> Option<usize> {
        self.positions
            .iter()
            .enumerate()
            .find_map(|(i, position_details)| {
                if position_details.address == Pubkey::default()
                    && position_details.whirlpool == Pubkey::default()
                {
                    Some(i)
                } else {
                    None
                }
            })
    }

    /// Get the index of the position if it exists
    pub fn position_index(&self, position: Pubkey) -> Option<usize> {
        self.positions
            .iter()
            .enumerate()
            .find_map(|(i, position_details)| {
                // It is safe to only check the position as 2 whirlpools shouldn't have the same position address
                if position_details.address == position {
                    Some(i)
                } else {
                    None
                }
            })
    }

    /// Add a new position
    pub(crate) fn add_position(&mut self, position_details: PositionDetails) -> Result<()> {
        for slot in self.positions.iter_mut() {
            if slot.address == Pubkey::default() {
                *slot = position_details;
                return Ok(());
            }
        }

        msg!("No free position slots");
        err!(MarginOrcaErrorCode::PositionUpdateError)
    }

    /// Clear position at an index
    pub(crate) fn clear_position(&mut self, index: usize) -> Result<()> {
        assert!(index <= MAX_POSITIONS);
        let position = &self.positions[index];

        if position.address == Pubkey::default() {
            msg!("Cannot clear an already slot, is the slot correct?");
            return err!(MarginOrcaErrorCode::PositionUpdateError);
        }
        if position.liquidity > 0 {
            msg!("Cannot clear a position with liquidity");
            return err!(MarginOrcaErrorCode::PositionUpdateError);
        }
        self.positions[index] = Default::default();

        Ok(())
    }

    #[inline(always)]
    pub fn positions(&self) -> impl IntoIterator<Item = &PositionDetails> {
        self.positions
            .iter()
            .filter(|address| address.address != Pubkey::default())
    }

    #[inline(always)]
    fn positions_mut(&mut self) -> impl IntoIterator<Item = &mut PositionDetails> {
        self.positions
            .iter_mut()
            .filter(|address| address.address != Pubkey::default())
    }

    #[inline(always)]
    pub fn total_whirlpools(&self) -> usize {
        self.positions()
            .into_iter()
            .map(|p| p.whirlpool)
            .collect::<HashSet<_>>()
            .len()
    }

    /// Update the whirlpool prices for positions that belong to it
    pub(crate) fn update_whirlpool_prices(
        &mut self,
        whirlpool: &Account<Whirlpool>,
        timestamp: i64,
    ) {
        for position_details in self.positions_mut() {
            if position_details.whirlpool != whirlpool.key() {
                continue;
            }
            position_details.current_sqrt_price = whirlpool.sqrt_price;
            position_details.tick_index_current = whirlpool.tick_current_index;
            position_details.last_refresh = timestamp;
        }
    }

    pub(crate) fn update_position(&mut self, position: &Account<WhirlpoolPosition>) -> Result<()> {
        let index = self
            .position_index(position.key())
            .ok_or(MarginOrcaErrorCode::PositionUpdateError)?;
        let mut position_details = self.positions.get_mut(index).unwrap();

        position_details.liquidity = position.liquidity;
        position_details.fee_owed_a = position.fee_owed_a;
        position_details.fee_owed_b = position.fee_owed_b;

        Ok(())
    }

    pub(crate) fn update_oracle_prices(
        &mut self,
        oracle_a: &AccountInfo,
        oracle_b: &AccountInfo,
    ) -> Result<()> {
        let token_a_oracle = match pyth_sdk_solana::load_price_feed_from_account_info(oracle_a) {
            Ok(pf) => pf,
            Err(e) => {
                msg!("the oracle account is not valid: {:?}", e);
                return err!(MarginOrcaErrorCode::InvalidOracle);
            }
        };
        // TODO: DRY
        let token_b_oracle = match pyth_sdk_solana::load_price_feed_from_account_info(oracle_b) {
            Ok(pf) => pf,
            Err(e) => {
                msg!("the oracle account is not valid: {:?}", e);
                return err!(MarginOrcaErrorCode::InvalidOracle);
            }
        };

        // CHECK: This relies on the margin program verifying oracle staleness.
        // We return the date of the oldest oracle in the pair.
        let price_a = token_a_oracle.get_price_unchecked();
        let price_b = token_b_oracle.get_price_unchecked();

        self.price_a = Number128::from_decimal(price_a.price, price_a.expo).into_bits();
        self.price_b = Number128::from_decimal(price_b.price, price_b.expo).into_bits();
        self.price_a_timestamp = price_a.publish_time;
        self.price_b_timestamp = price_b.publish_time;

        Ok(())
    }

    /// Value the overall position in the context of tokens that the user would
    /// receive if they were to withdraw all their liquidity.
    /// The calculation is based on the latest oracle prices.
    pub fn position_token_balances(
        &self,
        whirlpool_config: &WhirlpoolConfig,
    ) -> Result<(u64, u64)> {
        let mut token_a = 0u64;
        let mut token_b = 0u64;

        // Get the oracle price and use that for pricing the position.
        // Using the price removes the dependency on the whirlpool's price, as that has a
        // lower probability of being updated as quickly as the oracle (esp in low volume pools).
        // If the user is withdrawing from the pool, we do not invoke this function,
        // and this is handled by the whirlpool. If there indeed is a significant difference between the
        // oracle and the pool, and that difference has not been arbitraged, it does not concern
        // our position(s), as the user could arb the difference themselves by taking an action
        // directly against the whirlpool.
        // We are here concerned with a user inflating their position's collateral weight, and using
        // that to exploit the rest of the protocol.
        let (oracle_tick_index, oracle_sqrt_price) = self.oracle_price(whirlpool_config)?;

        for position_details in self.positions() {
            // If a position has no liquidity, skip it
            if position_details.liquidity == 0 {
                continue;
            }
            // Calculate the number of entitled tokens under 3 scenarios:
            // 1. The current oracle price
            // 2. The lower tick index
            // 3. The upper tick index
            // After calculating the tokens, find the tokens such that the position's USD value is lowest.

            let liquidity_delta = TryInto::<i128>::try_into(position_details.liquidity)
                .map_err(|_| MarginOrcaErrorCode::ArithmeticError)?;
            // Reconstruct a whirlpool position with the bare info required
            let position = WhirlpoolPosition {
                whirlpool: position_details.whirlpool,
                liquidity: position_details.liquidity,
                tick_lower_index: position_details.tick_index_lower,
                tick_upper_index: position_details.tick_index_upper,
                ..Default::default()
            };

            let lower_sqrt_price = sqrt_price_from_tick_index(position_details.tick_index_lower);
            let upper_sqrt_price = sqrt_price_from_tick_index(position_details.tick_index_upper);

            // First at the oracle price
            let (mut a, mut b, mut c) = self.position_min_value(
                &position,
                whirlpool_config,
                oracle_tick_index,
                oracle_sqrt_price,
                -liquidity_delta,
            )?;
            // Then at the lower tick index
            let lower = self.position_min_value(
                &position,
                whirlpool_config,
                position_details.tick_index_lower,
                lower_sqrt_price,
                -liquidity_delta,
            )?;
            if lower.2 < c {
                a = lower.0;
                b = lower.1;
                c = lower.2;
            }
            // Then at the upper tick index
            let upper = self.position_min_value(
                &position,
                whirlpool_config,
                position_details.tick_index_upper,
                upper_sqrt_price,
                -liquidity_delta,
            )?;
            if upper.2 < c {
                a = upper.0;
                b = upper.1;
            }

            // Add the worst valuation tokens to the total
            token_a = token_a
                .checked_add(a)
                .ok_or(MarginOrcaErrorCode::ArithmeticError)?;
            token_b = token_b
                .checked_add(b)
                .ok_or(MarginOrcaErrorCode::ArithmeticError)?;

            // Add fees at their currently accrued tokens
            token_a = token_a
                .checked_add(position_details.fee_owed_a)
                .ok_or(MarginOrcaErrorCode::ArithmeticError)?;
            token_b = token_b
                .checked_add(position_details.fee_owed_b)
                .ok_or(MarginOrcaErrorCode::ArithmeticError)?;
        }

        Ok((token_a, token_b))
    }

    pub fn update_position_balance(
        &self,
        margin_account: &MarginAccount,
        whirlpool_config: &WhirlpoolConfig,
    ) -> Result<()> {
        // The publish timestamp is the earliest timestamp of all positions and oracles
        let publish_time = self.price_a_timestamp.min(self.price_b_timestamp);
        let position_update_times = self.positions().into_iter().map(|p| p.last_refresh).min();
        let earliest_refresh = publish_time.min(position_update_times.unwrap_or(publish_time));

        let (token_balance_a, token_balance_b) = self.position_token_balances(whirlpool_config)?;

        // Calculate the weighted value of both tokens
        let balance_a =
            Number128::from_decimal(token_balance_a, -(whirlpool_config.mint_a_decimals as i32));
        let balance_b =
            Number128::from_decimal(token_balance_b, -(whirlpool_config.mint_b_decimals as i32));

        let value_a = balance_a.safe_mul(Number128::from_bits(self.price_a))?;
        let value_b = balance_b.safe_mul(Number128::from_bits(self.price_b))?;
        let total_value = value_a + value_b;
        // We divide by 1 to prevent an overflow issue
        let unit_value = total_value.safe_div(Number128::ONE)?;

        let unit_value_i64: i64 =
            i64::try_from(unit_value.as_u64(POSITION_VALUE_EXPO)).expect("Value overflowed");

        jet_margin::write_adapter_result(
            margin_account,
            &AdapterResult {
                position_changes: vec![(
                    whirlpool_config.position_mint,
                    vec![PositionChange::Price(PriceChangeInfo {
                        publish_time: earliest_refresh,
                        exponent: POSITION_VALUE_EXPO,
                        value: unit_value_i64,
                        confidence: 0,        // TODO
                        twap: unit_value_i64, // TODO
                    })],
                )],
            },
        )
    }

    fn oracle_price(&self, whirlpool_config: &WhirlpoolConfig) -> Result<(i32, u128)> {
        let price_a = Number128::from_bits(self.price_a);
        let price_b = Number128::from_bits(self.price_b);

        let pair_price = price_a
            .safe_div(price_b)?
            .safe_mul(Number128::ONE)?
            .as_f64();

        // In a SOL/USDC pair where SOL decimals = 9 and USDC = 6, expo = 0.001;
        let expo = 10f64.powi(
            whirlpool_config.mint_b_decimals as i32 - whirlpool_config.mint_a_decimals as i32,
        );
        // Get the pool's current tick index based on the oracle price.
        let tick_index = f64::log(pair_price * expo, 1.0001).round() as i32;

        Ok((tick_index, sqrt_price_from_tick_index(tick_index)))
    }

    fn position_min_value(
        &self,
        position: &WhirlpoolPosition,
        whirlpool_config: &WhirlpoolConfig,
        current_tick_index: i32,
        current_sqrt_price: u128,
        liquidity_delta: i128,
    ) -> Result<(u64, u64, u64)> {
        let (tokens_a, tokens_b) = calculate_liquidity_token_deltas(
            current_tick_index,
            current_sqrt_price,
            position,
            -liquidity_delta,
        )?;

        let a = Number128::from_decimal(tokens_a, -(whirlpool_config.mint_a_decimals as i32));
        let b = Number128::from_decimal(tokens_b, -(whirlpool_config.mint_b_decimals as i32));

        let value_a = a
            .safe_mul(Number128::from_bits(self.price_a))?
            .safe_div(Number128::ONE)?;
        let value_b = b
            .safe_mul(Number128::from_bits(self.price_b))?
            .safe_div(Number128::ONE)?;

        let total_value = value_a + value_b;
        let value = total_value.as_u64(POSITION_VALUE_EXPO);

        Ok((tokens_a, tokens_b, value))
    }
}
