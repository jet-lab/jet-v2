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

use anchor_lang::{prelude::*, solana_program::clock::UnixTimestamp};
use jet_proto_math::Number;
use pyth_sdk_solana::PriceFeed;
#[cfg(any(test, feature = "cli"))]
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::{
    borrow::{Borrow, BorrowMut},
    cmp::Ordering,
};

use crate::{
    util::{self, supply},
    Amount, AmountKind, ChangeKind, ErrorCode, TokenChange,
};

/// Account containing information about a margin pool, which
/// services lending/borrowing operations.
#[account]
#[repr(C, align(8))]
#[derive(Debug, Default)]
pub struct MarginPool {
    pub version: u8,

    /// The bump seed used to create the pool address
    pub pool_bump: [u8; 1],

    /// The address of the vault account, which has custody of the
    /// pool's tokens
    pub vault: Pubkey,

    /// The address of the account to deposit collected fees, represented as
    /// deposit notes
    pub fee_destination: Pubkey,

    /// The address of the mint for deposit notes
    pub deposit_note_mint: Pubkey,

    /// The address of the mint for the loan notes
    pub loan_note_mint: Pubkey,

    /// The token the pool allows lending and borrowing on
    pub token_mint: Pubkey,

    /// The address of the pyth oracle with price information for the token
    pub token_price_oracle: Pubkey,

    /// The address of this pool
    pub address: Pubkey,

    /// The configuration of the pool
    pub config: MarginPoolConfig,

    /// The total amount of tokens borrowed, that need to be repaid to
    /// the pool.
    pub borrowed_tokens: [u8; 24],

    /// The total amount of tokens in the pool that's reserved for collection
    /// as fees.
    pub uncollected_fees: [u8; 24],

    /// The time the interest was last accrued up to
    pub accrued_until: i64,
}

#[cfg(any(test, feature = "cli"))]
impl Serialize for MarginPool {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("MarginPool", 13)?;
        s.serialize_field("version", &self.version)?;
        s.serialize_field("vault", &self.vault.to_string())?;
        s.serialize_field("feeDestination", &self.fee_destination.to_string())?;
        s.serialize_field("depositNoteMint", &self.deposit_note_mint.to_string())?;
        s.serialize_field("loanNoteMint", &self.loan_note_mint.to_string())?;
        s.serialize_field("tokenMint", &self.token_mint.to_string())?;
        s.serialize_field("tokenPriceOracle", &self.token_price_oracle.to_string())?;
        s.serialize_field("borrowedTokens", &self.total_borrowed().to_string())?;
        s.serialize_field(
            "uncollectedFees",
            &self.total_uncollected_fees().to_string(),
        )?;
        s.serialize_field("accruedUntil", &self.accrued_until)?;
        s.end()
    }
}

impl MarginPool {
    /// Get the seeds needed to sign for the vault
    pub fn signer_seeds(&self) -> Result<[&[u8]; 2]> {
        if self.flags().contains(PoolFlags::DISABLED) {
            msg!("the pool is currently disabled");
            return err!(ErrorCode::Disabled);
        }

        Ok([self.token_mint.as_ref(), self.pool_bump.as_ref()])
    }

    /// Record a loan from the pool
    pub fn borrow(&mut self, tokens: u64) -> Result<()> {
        if !self.flags().contains(PoolFlags::ALLOW_LENDING) {
            msg!("this pool only allows deposits");
            return err!(ErrorCode::DepositsOnly);
        }

        *self.total_borrowed_mut() += Number::from(tokens);

        Ok(())
    }

    /// Record a repayment of a loan
    pub fn repay(&mut self, tokens: u64) -> Result<()> {
        // Due to defensive rounding, and probably only when the final outstanding loan in a pool
        // is being repaid, it is possible that the integer number of tokens being repaid exceeds
        // the precise number of total borrowed tokens. To cover this case, we guard against any
        // difference beyond the rounding effect, and use a saturating sub to update the total borrowed.

        if self.total_borrowed().as_u64_ceil(0) < tokens {
            return Err(ErrorCode::RepaymentExceedsTotalOutstanding.into());
        }

        *self.total_borrowed_mut() = self.total_borrowed().saturating_sub(Number::from(tokens));

        Ok(())
    }

    fn total_uncollected_fees_mut(&mut self) -> &mut Number {
        bytemuck::from_bytes_mut(&mut self.uncollected_fees)
    }

    pub fn total_uncollected_fees(&self) -> &Number {
        bytemuck::from_bytes(&self.uncollected_fees)
    }

    fn total_borrowed_mut(&mut self) -> &mut Number {
        bytemuck::from_bytes_mut(&mut self.borrowed_tokens)
    }

    pub fn total_borrowed(&self) -> &Number {
        bytemuck::from_bytes(&self.borrowed_tokens)
    }

    fn flags(&self) -> PoolFlags {
        PoolFlags::from_bits_truncate(self.config.flags)
    }

    pub fn join(&self) -> PoolManager<&MarginPool, (), (), ()> {
        PoolManager {
            pool: self,
            vault: (),
            deposit_note_mint: (),
            loan_note_mint: (),
        }
    }

    pub fn join_mut(&mut self) -> PoolManager<&mut MarginPool, (), (), ()> {
        PoolManager {
            pool: self,
            vault: (),
            deposit_note_mint: (),
            loan_note_mint: (),
        }
    }
}

/// Combines a margin pool with optional accounts that contain relevant balances.
/// The type system is used to ensure at compile time that the accounts are
/// populated before any operations that would require the account are used.
/// Likewise it is critical that you do not allow this struct to be
/// instantiated without using the proper join_* or with_* builder methods.
/// So keep the AccountInfo fields private.
pub struct PoolManager<P, V, D, L> {
    pub pool: P,
    vault: V,
    deposit_note_mint: D,
    loan_note_mint: L,
}

/// These are builder/plug-in methods to reconstruct the PoolManager
/// with additional optional dependencies mixed in
/// They only require a pool for validations
impl<P: Borrow<MarginPool>, V, D, L> PoolManager<P, V, D, L> {
    pub fn with_vault<'info, A: ToAccountInfo<'info>>(
        self,
        vault: &A,
    ) -> PoolManager<P, AccountInfo<'info>, D, L> {
        let vault = vault.to_account_info();
        assert_eq!(&self.pool.borrow().vault, vault.key);
        PoolManager {
            pool: self.pool,
            vault,
            deposit_note_mint: self.deposit_note_mint,
            loan_note_mint: self.loan_note_mint,
        }
    }

    pub fn with_loan_note_mint<'info, A: ToAccountInfo<'info>>(
        self,
        mint: &A,
    ) -> PoolManager<P, V, D, AccountInfo<'info>> {
        let mint = mint.to_account_info();
        assert_eq!(&self.pool.borrow().loan_note_mint, mint.key);
        PoolManager {
            pool: self.pool,
            vault: self.vault,
            deposit_note_mint: self.deposit_note_mint,
            loan_note_mint: mint,
        }
    }

    pub fn with_deposit_note_mint<'info, A: ToAccountInfo<'info>>(
        self,
        mint: &A,
    ) -> PoolManager<P, V, AccountInfo<'info>, L> {
        let mint = mint.to_account_info();
        assert_eq!(&self.pool.borrow().deposit_note_mint, mint.key);
        PoolManager {
            pool: self.pool,
            vault: self.vault,
            deposit_note_mint: mint,
            loan_note_mint: self.loan_note_mint,
        }
    }
}

/// Requires vault
impl<'info, P, D, L> PoolManager<P, AccountInfo<'info>, D, L> {
    pub fn vault_balance(&self) -> Result<u64> {
        anchor_spl::token::accessor::amount(&self.vault)
    }
}

/// Requires mutable pool and vault
impl<'info, P: BorrowMut<MarginPool>, D, L> PoolManager<P, AccountInfo<'info>, D, L> {
    /// Accrue interest charges on outstanding borrows
    ///
    /// Returns true if the interest was fully accumulated, false if it was
    /// only partially accumulated (due to significant time drift).
    pub fn accrue_interest(&mut self, time: UnixTimestamp) -> Result<bool> {
        let pool = self.pool.borrow();
        let time_behind = time - pool.accrued_until;
        let time_to_accrue = std::cmp::min(time_behind, util::MAX_ACCRUAL_SECONDS);

        match time_to_accrue.cmp(&0) {
            Ordering::Less => panic!("Interest may not be accrued over a negative time period."),
            Ordering::Equal => Ok(true),
            Ordering::Greater => {
                let interest_rate = self.interest_rate()?;
                let compound_rate = util::compound_interest(interest_rate, time_to_accrue);
                let pool = self.pool.borrow_mut();

                let interest_fee_rate = Number::from_bps(pool.config.management_fee_rate);
                let new_interest_accrued = *pool.total_borrowed() * compound_rate;
                let fee_to_collect = new_interest_accrued * interest_fee_rate;

                *pool.total_borrowed_mut() += new_interest_accrued;
                *pool.total_uncollected_fees_mut() += fee_to_collect;

                pool.accrued_until = pool.accrued_until.checked_add(time_to_accrue).unwrap();

                Ok(time_behind == time_to_accrue)
            }
        }
    }
}

/// Requires pool and vault
impl<'info, P: Borrow<MarginPool>, D, L> PoolManager<P, AccountInfo<'info>, D, L> {
    pub fn total_value(&self) -> Result<Number> {
        Ok(*self.pool.borrow().total_borrowed() + Number::from(self.vault_balance()?))
    }

    /// Gets the current utilization rate of the pool
    pub fn utilization_rate(&self) -> Result<Number> {
        Ok(*self.pool.borrow().total_borrowed() / self.total_value()?)
    }

    /// Gets the current interest rate for loans from this pool
    pub fn interest_rate(&self) -> Result<Number> {
        let pool = self.pool.borrow();
        let borrow_1 = Number::from_bps(pool.config.borrow_rate_1);

        // Catch the edge case of empty pool
        if pool.total_borrowed() == &Number::ZERO {
            return Ok(borrow_1);
        }

        let util_rate = self.utilization_rate()?;

        let util_1 = Number::from_bps(pool.config.utilization_rate_1);

        if util_rate <= util_1 {
            // First regime
            let borrow_0 = Number::from_bps(pool.config.borrow_rate_0);

            return Ok(util::interpolate(
                util_rate,
                Number::ZERO,
                util_1,
                borrow_0,
                borrow_1,
            ));
        }

        let util_2 = Number::from_bps(pool.config.utilization_rate_2);
        let borrow_2 = Number::from_bps(pool.config.borrow_rate_2);

        if util_rate <= util_2 {
            // Second regime
            let borrow_1 = Number::from_bps(pool.config.borrow_rate_1);

            return Ok(util::interpolate(
                util_rate, util_1, util_2, borrow_1, borrow_2,
            ));
        }

        let borrow_3 = Number::from_bps(pool.config.borrow_rate_3);

        if util_rate < Number::ONE {
            // Third regime
            return Ok(util::interpolate(
                util_rate,
                util_2,
                Number::ONE,
                borrow_2,
                borrow_3,
            ));
        }

        // Maximum interest
        Ok(borrow_3)
    }
}

/// Requires pool, vault, and deposit note mint
impl<'info, P: Borrow<MarginPool>, L> PoolManager<P, AccountInfo<'info>, AccountInfo<'info>, L> {
    pub fn deposit_amount(&self) -> Result<FullAmountCalculator> {
        Ok(FullAmountCalculator {
            note_type: TokenType::DepositNote,
            total_tokens: self.total_value()? - *self.pool.borrow().total_uncollected_fees(),
            note_supply: supply(&self.deposit_note_mint)?.into(),
        })
    }
}

/// Requires pool and loan note mint
impl<'info, P: Borrow<MarginPool>, V, D> PoolManager<P, V, D, AccountInfo<'info>> {
    pub fn loan_amount(&self) -> Result<FullAmountCalculator> {
        Ok(FullAmountCalculator {
            note_type: TokenType::LoanNote,
            total_tokens: *self.pool.borrow().total_borrowed(),
            note_supply: supply(&self.loan_note_mint)?.into(),
        })
    }
}

/// Requires vault, deposit note mint, and loan note mint
impl<'info, P: Borrow<MarginPool>>
    PoolManager<P, AccountInfo<'info>, AccountInfo<'info>, AccountInfo<'info>>
{
    /// Calculate the prices for the deposit and loan notes, based on
    /// the price of the underlying token.
    pub fn calculate_prices(&self, pyth_price: &PriceFeed) -> Result<PriceResult> {
        let price_obj = pyth_price
            .get_current_price()
            .ok_or(ErrorCode::InvalidPrice)?;
        let ema_obj = pyth_price.get_ema_price().ok_or(ErrorCode::InvalidPrice)?;

        let price_value = Number::from_decimal(price_obj.price, price_obj.expo);
        let conf_value = Number::from_decimal(price_obj.conf, price_obj.expo);
        let twap_value = Number::from_decimal(ema_obj.price, ema_obj.expo);

        let deposit_note_exchange_rate = self.deposit_amount()?.exchange_rate();
        let loan_note_exchange_rate = self.loan_amount()?.exchange_rate();

        let deposit_note_price =
            (price_value * deposit_note_exchange_rate).as_u64_rounded(pyth_price.expo) as i64;
        let deposit_note_conf =
            (conf_value * deposit_note_exchange_rate).as_u64_rounded(pyth_price.expo) as u64;
        let deposit_note_twap =
            (twap_value * deposit_note_exchange_rate).as_u64_rounded(pyth_price.expo) as i64;
        let loan_note_price =
            (price_value * loan_note_exchange_rate).as_u64_rounded(pyth_price.expo) as i64;
        let loan_note_conf =
            (conf_value * loan_note_exchange_rate).as_u64_rounded(pyth_price.expo) as u64;
        let loan_note_twap =
            (twap_value * loan_note_exchange_rate).as_u64_rounded(pyth_price.expo) as i64;

        Ok(PriceResult {
            deposit_note_price,
            deposit_note_conf,
            deposit_note_twap,
            loan_note_price,
            loan_note_conf,
            loan_note_twap,
        })
    }
}

/// Requires mutable pool and deposit notes
impl<'info, P: BorrowMut<MarginPool>, L> PoolManager<P, AccountInfo<'info>, AccountInfo<'info>, L> {
    /// Collect any fees accumulated from interest
    ///
    /// Returns the number of notes to mint to represent the collected fees
    pub fn collect_accrued_fees(&mut self) -> Result<u64> {
        let pool = self.pool.borrow();
        let threshold = Number::from(pool.config.management_fee_collect_threshold);
        let uncollected = *pool.total_uncollected_fees();

        if uncollected < threshold {
            // not enough accumulated to be worth minting new notes
            return Ok(0);
        }

        let fee_notes = (uncollected / self.deposit_amount()?.exchange_rate()).as_u64(0);

        *self.pool.borrow_mut().total_uncollected_fees_mut() = Number::ZERO;

        Ok(fee_notes)
    }
}

pub struct FullAmountCalculator {
    note_type: TokenType,
    pub total_tokens: Number,
    pub note_supply: Number,
}

impl FullAmountCalculator {
    pub fn from_request(
        &self,
        current_notes: u64,
        change: TokenChange,
        action: PoolAction,
    ) -> Result<FullAmount> {
        let full_amount = self.from_amount(Amount::tokens(change.tokens));
        match change.kind {
            ChangeKind::ShiftBy => full_amount,
            ChangeKind::SetTo => self.from_set_amount(current_notes, full_amount?.notes, action),
        }
    }

    fn from_set_amount(
        &self,
        current_notes: u64,
        target_notes: Number,
        pool_action: PoolAction,
    ) -> Result<FullAmount> {
        let delta = match pool_action {
            PoolAction::Borrow | PoolAction::Deposit => target_notes - Number::from(current_notes),
            PoolAction::Withdraw | PoolAction::Repay => Number::from(current_notes) - target_notes,
        };

        Ok(self.from_notes(delta))
    }

    /// Convert the `Amount` to a `FullAmount` conisting of the appropriate proprtion of notes and tokens
    fn from_amount(&self, amount: Amount) -> Result<FullAmount> {
        let amount = self.full_amount(amount);

        // As FullAmount represents the conversion of tokens to/from notes for
        // the purpose of:
        // - adding/subtracting tokens to/from a pool's vault
        // - minting/burning notes from a pool's deposit/loan mint.
        // There should be no scenario where a conversion between notes and tokens
        // leads to either value being 0 while the other is not.
        //
        // Scenarios where this can happen could be security risks, such as:
        // - A user withdraws 1 token but burns 0 notes, they are draining the pool.
        // - A user deposits 1 token but mints 0 notes, they are losing funds for no value.
        // - A user deposits 0 tokens but mints 1 notes, they are getting free deposits.
        // - A user withdraws 0 tokens but burns 1 token, they are writing off debt.
        //
        // Thus we finally check that both values are positive.
        if (amount.notes == Number::ZERO && amount.tokens > Number::ZERO)
            || (amount.tokens == Number::ZERO && amount.notes > Number::ZERO)
        {
            return err!(crate::ErrorCode::InvalidAmount);
        }

        Ok(amount)
    }

    fn full_amount(&self, amount: Amount) -> FullAmount {
        match amount.kind {
            AmountKind::Tokens => FullAmount {
                note_type: self.note_type,
                tokens: Number::from(amount.value),
                notes: Number::from(amount.value) / self.exchange_rate(),
            },

            AmountKind::Notes => FullAmount {
                note_type: self.note_type,
                notes: Number::from(amount.value),
                tokens: Number::from(amount.value) * self.exchange_rate(),
            },
        }
    }

    pub fn from_notes(&self, amount: Number) -> FullAmount {
        FullAmount {
            note_type: self.note_type,
            notes: amount,
            tokens: amount * self.exchange_rate(),
        }
    }

    pub fn from_tokens(&self, amount: Number) -> FullAmount {
        FullAmount {
            note_type: self.note_type,
            tokens: amount,
            notes: amount / self.exchange_rate(),
        }
    }

    /// Get the exchange rate for note -> token
    /// - total_tokens: total number of tokens that are represented by the notes
    /// - note_supply: total supply of notes that exist
    fn exchange_rate(&self) -> Number {
        let notes = std::cmp::max(Number::ONE, self.note_supply);
        let total_borrowed = std::cmp::max(Number::ONE, self.total_tokens);

        total_borrowed / notes
    }
}

#[derive(Debug)]
pub struct FullAmount {
    note_type: TokenType,
    pub tokens: Number,
    pub notes: Number,
}

impl FullAmount {
    pub fn as_token_transfer(&self, direction: TransferDirection) -> u64 {
        self.tokens.as_transfer(TokenType::Underlying, direction)
    }

    pub fn as_note_transfer(&self, direction: TransferDirection) -> u64 {
        self.tokens.as_transfer(self.note_type.into(), direction)
    }
}

/// Represents the primary pool actions, used in determining the
/// rounding direction between tokens and notes.
#[derive(Clone, Copy)]
pub enum PoolAction {
    Borrow,
    Deposit,
    Repay,
    Withdraw,
}

/// Represents the direction in which we should round when converting
/// between tokens and notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundingDirection {
    Down,
    Up,
}

impl RoundingDirection {
    /// When value moves to the user, round down
    /// When value moves from the user, round up
    /// LoanNote has negative value, others are positive
    pub const fn for_transfer(token: TokenType, direction: TransferDirection) -> Self {
        use {RoundingDirection::*, TokenType::*, TransferDirection::*};
        match (token, direction) {
            (Underlying, FromUser) => Up,
            (Underlying, ToUser) => Down,
            (DepositNote, FromUser) => Up,
            (DepositNote, ToUser) => Down,
            (LoanNote, FromUser) => Down,
            (LoanNote, ToUser) => Up,
        }
    }
}

pub trait Round<T> {
    fn round(&self, direction: RoundingDirection) -> T;
}

impl Round<u64> for Number {
    fn round(&self, direction: RoundingDirection) -> u64 {
        match direction {
            RoundingDirection::Down => self.as_u64(0),
            RoundingDirection::Up => self.as_u64_ceil(0),
        }
    }
}

pub trait AsTransfer: Round<u64> {
    fn as_transfer(&self, token_type: TokenType, direction: TransferDirection) -> u64 {
        self.round(RoundingDirection::for_transfer(token_type, direction))
    }
}

impl<T: Round<u64>> AsTransfer for T {}

pub enum TransferDirection {
    ToUser,
    FromUser,
}
pub use TransferDirection::*;

#[derive(Debug, Clone, Copy)]
pub enum TokenType {
    Underlying,
    DepositNote,
    LoanNote,
}

pub struct PriceResult {
    pub deposit_note_price: i64,
    pub deposit_note_conf: u64,
    pub deposit_note_twap: i64,
    pub loan_note_price: i64,
    pub loan_note_conf: u64,
    pub loan_note_twap: i64,
}

/// Configuration for a margin pool
#[derive(Debug, Default, AnchorDeserialize, AnchorSerialize, Clone, Eq, PartialEq)]
pub struct MarginPoolConfig {
    /// Space for binary settings
    pub flags: u64,

    /// The utilization rate at which first regime transitions to second
    pub utilization_rate_1: u16,

    /// The utilization rate at which second regime transitions to third
    pub utilization_rate_2: u16,

    /// The lowest borrow rate
    pub borrow_rate_0: u16,

    /// The borrow rate at the transition point from first to second regime
    pub borrow_rate_1: u16,

    /// The borrow rate at the transition point from second to third regime
    pub borrow_rate_2: u16,

    /// The highest possible borrow rate.
    pub borrow_rate_3: u16,

    /// The fee rate applied to interest payments collected
    pub management_fee_rate: u16,

    /// The threshold for fee collection
    pub management_fee_collect_threshold: u64,
}

bitflags::bitflags! {
    pub struct PoolFlags: u64 {
        /// The pool is not allowed to sign for anything, preventing
        /// the movement of funds.
        const DISABLED = 1 << 0;

        /// The pool is allowed to lend out deposits for borrowing
        const ALLOW_LENDING = 1 << 1;
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use serde_test::{assert_ser_tokens, Token};

//     #[test]
//     fn test_deposit_note_rounding() -> Result<()> {
//         let mut margin_pool = MarginPool::default();

//         margin_pool.deposit(&FullAmount {
//             tokens: 1_000_000,
//             notes: 900_000,
//         });

//         // Deposit note exchange rate is 1.111111_.
//         // If a user withdraws 9 notes, they should get 9 or 10 tokens back
//         // depending on the rounding.

//         assert_eq!(
//             margin_pool.deposit_note_exchange_rate().as_u64(-9),
//             1111111111
//         );

//         let pool_convert = |amount, rounding| {
//             let exchange_rate = margin_pool.deposit_note_exchange_rate();
//             MarginPool::convert_with_rounding_and_rate(amount, rounding, exchange_rate)
//         };

//         let deposit_amount = pool_convert(Amount::notes(12), RoundingDirection::Down);

//         assert_eq!(deposit_amount.notes, 12);
//         assert_eq!(deposit_amount.tokens, 13); // ref [0]

//         let deposit_amount = pool_convert(Amount::notes(18), RoundingDirection::Down);

//         assert_eq!(deposit_amount.notes, 18);
//         assert_eq!(deposit_amount.tokens, 19);

//         let deposit_amount = pool_convert(Amount::notes(12), RoundingDirection::Up);

//         assert_eq!(deposit_amount.notes, 12);
//         assert_eq!(deposit_amount.tokens, 14); // ref [1]

//         // A user requesting 1 note should never get 0 tokens back,
//         // or 1 token should never get 0 notes back

//         let deposit_amount = pool_convert(Amount::notes(1), RoundingDirection::Down);

//         // When depositing, 1:1 would be advantageous to the user
//         assert_eq!(deposit_amount.notes, 1);
//         assert_eq!(deposit_amount.tokens, 1);

//         let deposit_amount = pool_convert(Amount::notes(1), RoundingDirection::Up);

//         // Depositing 2 tokens for 1 note is disadvantageous to the user
//         // and protects the protocol's average exchange rate
//         assert_eq!(deposit_amount.notes, 1);
//         assert_eq!(deposit_amount.tokens, 2);

//         // Check the default rounding for depositing notes, as it is disadvantageous
//         // to the user per the previous observation.
//         let direction = RoundingDirection::direction(PoolAction::Deposit, AmountKind::Notes);
//         assert_eq!(RoundingDirection::Up, direction);

//         // A repay is the same as a deposit (inflow)
//         let direction = RoundingDirection::direction(PoolAction::Repay, AmountKind::Notes);
//         assert_eq!(RoundingDirection::Up, direction);

//         Ok(())
//     }

//     /// Conversion between tokens and notes would allow a user to
//     /// provide tokens for notes, or to specify the number of tokens
//     /// to receive on withdrawal.
//     ///
//     /// As the exchange rate between notes and tokens is expected to
//     /// increase over time, there is a risk that a user could extract
//     /// 1 token while burning 0 notes due to rounding.
//     #[test]
//     fn test_deposit_token_rounding() -> Result<()> {
//         let mut margin_pool = MarginPool::default();

//         margin_pool.deposit(&FullAmount {
//             tokens: 1_000_000,
//             notes: 900_000,
//         });

//         assert_eq!(
//             margin_pool.deposit_note_exchange_rate().as_u64(-9),
//             1111111111
//         );

//         let pool_convert = |amount, rounding| {
//             let exchange_rate = margin_pool.deposit_note_exchange_rate();
//             MarginPool::convert_with_rounding_and_rate(amount, rounding, exchange_rate)
//         };

//         // depositing tokens should round down
//         let deposit_result = margin_pool.convert_amount(Amount::tokens(1), PoolAction::Deposit);

//         // Rounding down would return 0 notes
//         assert!(deposit_result.is_err());

//         let deposit_amount = pool_convert(Amount::tokens(1), RoundingDirection::Up);

//         // Depositing 1 token for 1 note is disadvantageous to the user as they
//         // get a lower rate than the 1.111_.
//         // This is however because they are requesting the smallest unit, so
//         // this test hides the true intention of the rounding.
//         assert_eq!(deposit_amount.notes, 1);
//         assert_eq!(deposit_amount.tokens, 1);

//         // It is better observed with a bigger number.
//         // The expectation when a user deposits is that they should get less notes
//         // than the exchange rate if we have to round. This is because fewer notes
//         // entitle the user to fewer tokens on withdrawal from the pool.

//         // We start by rounding up a bigger number. See [0]
//         let deposit_amount = pool_convert(Amount::tokens(9), RoundingDirection::Up);

//         assert_eq!(deposit_amount.notes, 9);
//         assert_eq!(deposit_amount.tokens, 9);

//         // [1] shows the behaviour when rounding 12 notes up, we get 13 tokens.
//         let deposit_amount = pool_convert(Amount::tokens(13), RoundingDirection::Up);

//         assert_eq!(deposit_amount.tokens, 13);
//         // [1] returned 12 notes, and we get 12 notes back.
//         assert_eq!(deposit_amount.notes, 12);

//         // If we round down instead of up, we preserve value.
//         let deposit_amount = pool_convert(Amount::tokens(14), RoundingDirection::Down);

//         assert_eq!(deposit_amount.tokens, 14);
//         assert_eq!(deposit_amount.notes, 12);

//         // From the above scenarios, we achieve a roundtrip when we change the
//         // rounding direction depending on the conversion direction.
//         // When depositing notes, we rounded up. When depositing tokens, rounding
//         // down leaves the user in a comparable scenario.

//         // Thus when depositing tokens, we should round down.
//         let direction = RoundingDirection::direction(PoolAction::Deposit, AmountKind::Tokens);
//         assert_eq!(RoundingDirection::Down, direction);

//         // Repay should behave like deposit
//         let direction = RoundingDirection::direction(PoolAction::Repay, AmountKind::Tokens);
//         assert_eq!(RoundingDirection::Down, direction);

//         Ok(())
//     }

//     #[test]
//     fn test_loan_note_rounding() -> Result<()> {
//         let mut margin_pool = MarginPool::default();
//         margin_pool.config.flags = PoolFlags::ALLOW_LENDING.bits();

//         // Deposit funds so there is liquidity
//         margin_pool.deposit(&FullAmount {
//             tokens: 1_000_000,
//             notes: 1_000_000,
//         });

//         margin_pool.borrow(&FullAmount {
//             tokens: 1_000_000,
//             notes: 900_000,
//         })?;

//         assert_eq!(margin_pool.loan_note_exchange_rate().as_u64(-9), 1111111111);

//         let pool_convert = |amount, rounding| {
//             let exchange_rate = margin_pool.loan_note_exchange_rate();
//             MarginPool::convert_with_rounding_and_rate(amount, rounding, exchange_rate)
//         };

//         let loan_amount = pool_convert(Amount::notes(1), RoundingDirection::Down);

//         assert_eq!(loan_amount.notes, 1);
//         assert_eq!(loan_amount.tokens, 1);

//         let loan_amount = pool_convert(Amount::notes(1), RoundingDirection::Up);

//         // When withdrawing, rounding up benefits the user at the cost of the
//         // protocol. The user gets to borrow at a lower rate (0.5 vs 1.111_).
//         assert_eq!(loan_amount.notes, 1);
//         assert_eq!(loan_amount.tokens, 2);

//         // Check that borrow rounding is down, so the user does not borrow at
//         // a lower rate.
//         let direction = RoundingDirection::direction(PoolAction::Withdraw, AmountKind::Notes);
//         assert_eq!(RoundingDirection::Down, direction);

//         // A borrow is the same as withdraw (outflow)
//         let direction = RoundingDirection::direction(PoolAction::Borrow, AmountKind::Notes);
//         assert_eq!(RoundingDirection::Down, direction);

//         Ok(())
//     }

//     #[test]
//     fn test_loan_token_rounding() -> Result<()> {
//         let mut margin_pool = MarginPool::default();
//         margin_pool.config.flags = PoolFlags::ALLOW_LENDING.bits();

//         margin_pool.deposit(&FullAmount {
//             tokens: 1_000_000,
//             notes: 1_000_000,
//         });

//         margin_pool.borrow(&FullAmount {
//             tokens: 1_000_000,
//             notes: 900_000,
//         })?;

//         assert_eq!(margin_pool.loan_note_exchange_rate().as_u64(-9), 1111111111);

//         let pool_convert = |amount, rounding| {
//             let exchange_rate = margin_pool.loan_note_exchange_rate();
//             MarginPool::convert_with_rounding_and_rate(amount, rounding, exchange_rate)
//         };

//         // repaying tokens rounds down
//         let loan_result = margin_pool.convert_amount(Amount::tokens(1), PoolAction::Repay);

//         // Rounding down to 0 is not allowed
//         assert!(loan_result.is_err());

//         let loan_amount = pool_convert(Amount::tokens(1), RoundingDirection::Up);

//         // When withdrawing tokens, the user should get 111 tokens for 100 notes (or less)
//         // at the current exchange rate. A 1:1 is disadvantageous to the user
//         // as the user can borrow 111 times, and get 111 tokens for 111 notes,
//         // which if they borrowed at once, they could have received more tokens.
//         assert_eq!(loan_amount.notes, 1);
//         assert_eq!(loan_amount.tokens, 1);

//         let loan_amount = pool_convert(Amount::tokens(111), RoundingDirection::Up);

//         assert_eq!(loan_amount.tokens, 111);
//         // Even at a larger quantity, rounding up is still disadvantageous as
//         // the user borrows at a lower rate than the prevailing exchange rate.
//         assert_eq!(loan_amount.notes, 100);

//         // In this instance, there is a difference in rationale between borrowing
//         // and withdrawing.
//         // When borrowing, we mint loan notes, and would want to mint more notes
//         // for the same tokens if rounding is involved.
//         let direction = RoundingDirection::direction(PoolAction::Borrow, AmountKind::Tokens);
//         assert_eq!(RoundingDirection::Up, direction);

//         // When withdrawing from a deposit pool, we want to give the user
//         // less tokens for more notes.
//         // Thus the rounding in a withdrawal from tokens should be up,
//         // as 1 token would mean more notes.
//         let direction = RoundingDirection::direction(PoolAction::Withdraw, AmountKind::Tokens);
//         assert_eq!(RoundingDirection::Up, direction);

//         Ok(())
//     }

//     #[test]
//     fn margin_pool_serialization() {
//         let pool = MarginPool::default();
//         assert_ser_tokens(
//             &pool,
//             &[
//                 Token::Struct {
//                     name: "MarginPool",
//                     len: 13,
//                 },
//                 Token::Str("version"),
//                 Token::U8(0),
//                 Token::Str("vault"),
//                 Token::Str("11111111111111111111111111111111"),
//                 Token::Str("feeDestination"),
//                 Token::Str("11111111111111111111111111111111"),
//                 Token::Str("depositNoteMint"),
//                 Token::Str("11111111111111111111111111111111"),
//                 Token::Str("loanNoteMint"),
//                 Token::Str("11111111111111111111111111111111"),
//                 Token::Str("tokenMint"),
//                 Token::Str("11111111111111111111111111111111"),
//                 Token::Str("tokenPriceOracle"),
//                 Token::Str("11111111111111111111111111111111"),
//                 Token::Str("borrowedTokens"),
//                 Token::Str("0.0"),
//                 Token::Str("uncollectedFees"),
//                 Token::Str("0.0"),
//                 Token::Str("depositTokens"),
//                 Token::U64(0),
//                 Token::Str("depositNotes"),
//                 Token::U64(0),
//                 Token::Str("loanNotes"),
//                 Token::U64(0),
//                 Token::Str("accruedUntil"),
//                 Token::I64(0),
//                 Token::StructEnd,
//             ],
//         );
//     }
// }
