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
    /// The current sqrt price from the whirlpool
    pub current_sqrt_price: u128,
    /// The current tick index from the whirlpool
    pub tick_index_current: i32,
    pub tick_index_lower: i32,
    pub tick_index_upper: i32,
    pub last_refresh: i64,
    pub fee_owed_a: u64,
    pub fee_owed_b: u64,
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

    /// Set a position at an empty index
    pub(crate) fn set_position(
        &mut self,
        position_details: PositionDetails,
        index: usize,
    ) -> Result<()> {
        assert!(index <= MAX_POSITIONS);

        if position_details.address == Pubkey::default() {
            msg!("Should not set_position to a default address");
            return err!(MarginOrcaErrorCode::PositionUpdateError);
        }

        // Q: if this is called internaly only, should we worry about duplicate
        // positions?
        if self.position_index(position_details.address).is_some() {
            msg!("Cannot add a position more than once");
            return err!(MarginOrcaErrorCode::PositionUpdateError);
        }

        if self.positions[index].address != Pubkey::default() {
            msg!("Slot has a position, cannot overwrite");
            return err!(MarginOrcaErrorCode::PositionUpdateError);
        }
        self.positions[index] = position_details;

        Ok(())
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
        // TODO: Ensure that this condition is met
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
    // Q: any better option than a hashmap?
    pub fn position_token_balances(&self) -> Result<(u64, u64)> {
        let mut token_a = 0u64;
        let mut token_b = 0u64;

        for position_details in self.positions() {
            // If a position has no liquidity, skip it
            if position_details.liquidity == 0 {
                continue;
            }
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
            let (a, b) = calculate_liquidity_token_deltas(
                position_details.tick_index_current,
                position_details.current_sqrt_price,
                &position,
                -liquidity_delta,
            )?;
            token_a = token_a
                .checked_add(a)
                .ok_or(MarginOrcaErrorCode::ArithmeticError)?;
            token_b = token_b
                .checked_add(b)
                .ok_or(MarginOrcaErrorCode::ArithmeticError)?;

            // Add fees (not yet rewards)
            // TODO: there should be an adapter_invoke that calls orca::update_fees_and_rewards for each position
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

        // TODO: as part of this we still have to find the correct minimum collateral value
        let (token_balance_a, token_balance_b) = self.position_token_balances()?;

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
}
