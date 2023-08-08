use std::{collections::HashMap, convert::TryInto};

use anchor_lang::prelude::*;
use orca_whirlpool::{
    manager::liquidity_manager::calculate_liquidity_token_deltas, state::Position,
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

#[account]
#[derive(Default)]
pub struct PositionMetadata {
    pub owner: Pubkey,
    pub whirlpool_config: Pubkey,

    // Instead of storing just the position's pubkey, we could also
    // store its liquidity information so we don't have to load the
    // underlying whenever we want to value it
    pub positions: [PositionDetails; MAX_POSITIONS],
    // TODO: what other information would we like to track?
}

declare_account_size!(PositionMetadata, 716);

#[derive(Clone, Copy, Default, AnchorSerialize, AnchorDeserialize)]
pub struct PositionDetails {
    pub whirlpool: Pubkey,
    pub position: Pubkey,
}

impl PositionMetadata {
    pub fn free_position(&self) -> Option<usize> {
        self.positions.iter().enumerate().find_map(|(i, address)| {
            if address.position == Pubkey::default() && address.whirlpool == Pubkey::default() {
                Some(i)
            } else {
                None
            }
        })
    }

    /// Get the index of the position if it exists
    pub fn position_index(&self, position: Pubkey) -> Option<usize> {
        self.positions.iter().enumerate().find_map(|(i, address)| {
            // It is safe to only check the position as 2 whirlpools shouldn't have the same position address
            if address.position == position {
                Some(i)
            } else {
                None
            }
        })
    }

    /// Set a position at an empty index
    pub(crate) fn set_position(
        &mut self,
        position: Pubkey,
        whirlpool: Pubkey,
        index: usize,
    ) -> Result<()> {
        assert!(index <= MAX_POSITIONS);

        if position == Pubkey::default() {
            msg!("Should not set_position to a default address");
            return err!(MarginOrcaErrorCode::PositionUpdateError);
        }

        // Q: if this is called internaly only, should we worry about duplicate
        // positions?
        if self.position_index(position).is_some() {
            msg!("Cannot add a position more than once");
            return err!(MarginOrcaErrorCode::PositionUpdateError);
        }

        if self.positions[index].position != Pubkey::default() {
            msg!("Slot has a position, cannot overwrite");
            return err!(MarginOrcaErrorCode::PositionUpdateError);
        }
        self.positions[index] = PositionDetails {
            whirlpool,
            position,
        };

        Ok(())
    }

    /// Clear position at an index
    pub(crate) fn clear_position(&mut self, index: usize) -> Result<()> {
        assert!(index <= MAX_POSITIONS);

        if self.positions[index].position == Pubkey::default() {
            msg!("Cannot clear an already slot, is the slot correct?");
            return err!(MarginOrcaErrorCode::PositionUpdateError);
        }
        self.positions[index] = Default::default();

        Ok(())
    }

    #[inline(always)]
    pub fn positions(&self) -> impl IntoIterator<Item = &PositionDetails> {
        self.positions
            .iter()
            .filter(|address| address.position != Pubkey::default())
    }

    /// Value the overall position in the context of tokens that the user would
    /// receive if they were to withdraw all their liquidity.
    // Q: any better option than a hashmap?
    pub fn position_token_balances(
        &self,
        positions: &HashMap<Pubkey, PositionValuation>,
    ) -> Result<(u64, u64)> {
        let mut token_a = 0u64;
        let mut token_b = 0u64;

        for address in self.positions() {
            let valuation = positions
                .get(&address.position)
                .ok_or(MarginOrcaErrorCode::PositionUpdateError)?;
            // If a position has no liquidity, skip it
            if valuation.position.liquidity == 0 {
                continue;
            }
            let liquidity_delta = TryInto::<i128>::try_into(valuation.position.liquidity)
                .map_err(|_| MarginOrcaErrorCode::ArithmeticError)?;
            let (a, b) = calculate_liquidity_token_deltas(
                valuation.current_tick_index,
                valuation.sqrt_price,
                &valuation.position,
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
                .checked_add(valuation.position.fee_owed_a)
                .ok_or(MarginOrcaErrorCode::ArithmeticError)?;
            token_b = token_b
                .checked_add(valuation.position.fee_owed_b)
                .ok_or(MarginOrcaErrorCode::ArithmeticError)?;
        }

        Ok((token_a, token_b))
    }
}

pub struct PositionValuation {
    pub position: Position,
    pub current_tick_index: i32,
    pub sqrt_price: u128,
}
