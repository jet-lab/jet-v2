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
use std::cmp::Ordering;

use crate::{util, Amount, AmountKind, ErrorCode};

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

    /// The total amount of tokens available in the pool's vault
    pub deposit_tokens: u64,

    /// The total amount of notes issued to depositors of tokens.
    pub deposit_notes: u64,

    /// The total amount of notes issued to borrowers of tokens
    pub loan_notes: u64,

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
        s.serialize_field("depositTokens", &self.deposit_tokens)?;
        s.serialize_field("depositNotes", &self.deposit_notes)?;
        s.serialize_field("loanNotes", &self.loan_notes)?;
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

    /// Record a deposit into the pool
    pub fn deposit(&mut self, amount: &FullAmount) {
        self.deposit_tokens = self.deposit_tokens.checked_add(amount.tokens).unwrap();
        self.deposit_notes = self.deposit_notes.checked_add(amount.notes).unwrap();
    }

    /// Record a withdrawal from the pool
    pub fn withdraw(&mut self, amount: &FullAmount) -> Result<()> {
        self.deposit_tokens = self
            .deposit_tokens
            .checked_sub(amount.tokens)
            .ok_or(ErrorCode::InsufficientLiquidity)?;
        self.deposit_notes = self
            .deposit_notes
            .checked_sub(amount.notes)
            .ok_or(ErrorCode::InsufficientLiquidity)?;

        Ok(())
    }

    /// Record a loan from the pool
    pub fn borrow(&mut self, amount: &FullAmount) -> Result<()> {
        if !self.flags().contains(PoolFlags::ALLOW_LENDING) {
            msg!("this pool only allows deposits");
            return err!(ErrorCode::DepositsOnly);
        }

        self.deposit_tokens = self
            .deposit_tokens
            .checked_sub(amount.tokens)
            .ok_or(ErrorCode::InsufficientLiquidity)?;
        self.loan_notes = self.loan_notes.checked_add(amount.notes).unwrap();

        *self.total_borrowed_mut() += Number::from(amount.tokens);

        Ok(())
    }

    /// Record a repayment of a loan
    pub fn repay(&mut self, amount: &FullAmount) -> Result<()> {
        self.deposit_tokens = self.deposit_tokens.checked_add(amount.tokens).unwrap();
        self.loan_notes = self
            .loan_notes
            .checked_sub(amount.notes)
            .ok_or(ErrorCode::InsufficientLiquidity)?;

        // Due to defensive rounding, and probably only when the final outstanding loan in a pool
        // is being repaid, it is possible that the integer number of tokens being repaid exceeds
        // the precise number of total borrowed tokens. To cover this case, we guard against any
        // difference beyond the rounding effect, and use a saturating sub to update the total borrowed.

        if self.total_borrowed().as_u64_ceil(0) < amount.tokens {
            return Err(ErrorCode::RepaymentExceedsTotalOutstanding.into());
        }

        *self.total_borrowed_mut() = self
            .total_borrowed()
            .saturating_sub(Number::from(amount.tokens));

        Ok(())
    }

    /// Accrue interest charges on outstanding borrows
    ///
    /// Returns true if the interest was fully accumulated, false if it was
    /// only partially accumulated (due to significant time drift).
    pub fn accrue_interest(&mut self, time: UnixTimestamp) -> bool {
        let time_behind = time - self.accrued_until;
        let time_to_accrue = std::cmp::min(time_behind, util::MAX_ACCRUAL_SECONDS);

        match time_to_accrue.cmp(&0) {
            Ordering::Less => panic!("Interest may not be accrued over a negative time period."),
            Ordering::Equal => true,
            Ordering::Greater => {
                let interest_rate = self.interest_rate();
                let compound_rate = util::compound_interest(interest_rate, time_to_accrue);

                let interest_fee_rate = Number::from_bps(self.config.management_fee_rate);
                let new_interest_accrued = *self.total_borrowed() * compound_rate;
                let fee_to_collect = new_interest_accrued * interest_fee_rate;

                *self.total_borrowed_mut() += new_interest_accrued;
                *self.total_uncollected_fees_mut() += fee_to_collect;

                self.accrued_until = self.accrued_until.checked_add(time_to_accrue).unwrap();

                time_behind == time_to_accrue
            }
        }
    }

    /// Gets the current interest rate for loans from this pool
    pub fn interest_rate(&self) -> Number {
        let borrow_1 = Number::from_bps(self.config.borrow_rate_1);

        // Catch the edge case of empty pool
        if self.deposit_notes == 0 {
            return borrow_1;
        }

        let util_rate = self.utilization_rate();

        let util_1 = Number::from_bps(self.config.utilization_rate_1);

        if util_rate <= util_1 {
            // First regime
            let borrow_0 = Number::from_bps(self.config.borrow_rate_0);

            return util::interpolate(util_rate, Number::ZERO, util_1, borrow_0, borrow_1);
        }

        let util_2 = Number::from_bps(self.config.utilization_rate_2);
        let borrow_2 = Number::from_bps(self.config.borrow_rate_2);

        if util_rate <= util_2 {
            // Second regime
            let borrow_1 = Number::from_bps(self.config.borrow_rate_1);

            return util::interpolate(util_rate, util_1, util_2, borrow_1, borrow_2);
        }

        let borrow_3 = Number::from_bps(self.config.borrow_rate_3);

        if util_rate < Number::ONE {
            // Third regime
            return util::interpolate(util_rate, util_2, Number::ONE, borrow_2, borrow_3);
        }

        // Maximum interest
        borrow_3
    }

    /// Gets the current utilization rate of the pool
    pub fn utilization_rate(&self) -> Number {
        *self.total_borrowed() / self.total_value()
    }

    /// Collect any fees accumulated from interest
    ///
    /// Returns the number of notes to mint to represent the collected fees
    pub fn collect_accrued_fees(&mut self) -> u64 {
        let threshold = Number::from(self.config.management_fee_collect_threshold);
        let uncollected = *self.total_uncollected_fees();

        if uncollected < threshold {
            // not enough accumulated to be worth minting new notes
            return 0;
        }

        let fee_notes = (uncollected / self.deposit_note_exchange_rate()).as_u64(0);

        *self.total_uncollected_fees_mut() = Number::ZERO;
        self.deposit_notes = self.deposit_notes.checked_add(fee_notes).unwrap();

        fee_notes
    }

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

        let deposit_note_price = (price_value * self.deposit_note_exchange_rate())
            .as_u64_rounded(pyth_price.expo) as i64;
        let deposit_note_conf =
            (conf_value * self.deposit_note_exchange_rate()).as_u64_rounded(pyth_price.expo) as u64;
        let deposit_note_twap =
            (twap_value * self.deposit_note_exchange_rate()).as_u64_rounded(pyth_price.expo) as i64;
        let loan_note_price =
            (price_value * self.loan_note_exchange_rate()).as_u64_rounded(pyth_price.expo) as i64;
        let loan_note_conf =
            (conf_value * self.loan_note_exchange_rate()).as_u64_rounded(pyth_price.expo) as u64;
        let loan_note_twap =
            (twap_value * self.loan_note_exchange_rate()).as_u64_rounded(pyth_price.expo) as i64;

        Ok(PriceResult {
            deposit_note_price,
            deposit_note_conf,
            deposit_note_twap,
            loan_note_price,
            loan_note_conf,
            loan_note_twap,
        })
    }

    /// Convert the amount to be representable by tokens and notes for deposits
    pub fn convert_deposit_amount(
        &self,
        amount: Amount,
        rounding: RoundingDirection,
    ) -> Result<FullAmount> {
        self.convert_amount(amount, self.deposit_note_exchange_rate(), rounding)
    }

    /// Convert the amount to be representable by tokens and notes for borrows
    pub fn convert_loan_amount(
        &self,
        amount: Amount,
        rounding: RoundingDirection,
    ) -> Result<FullAmount> {
        self.convert_amount(amount, self.loan_note_exchange_rate(), rounding)
    }

    fn convert_amount(
        &self,
        amount: Amount,
        exchange_rate: Number,
        rounding: RoundingDirection,
    ) -> Result<FullAmount> {
        let amount = match amount.kind {
            AmountKind::Tokens => FullAmount {
                tokens: amount.value,
                notes: match rounding {
                    RoundingDirection::Down => {
                        (Number::from(amount.value) / exchange_rate).as_u64(0)
                    }
                    RoundingDirection::Up => {
                        (Number::from(amount.value) / exchange_rate).as_u64_ceil(0)
                    }
                },
            },

            AmountKind::Notes => FullAmount {
                notes: amount.value,
                tokens: match rounding {
                    RoundingDirection::Down => {
                        (Number::from(amount.value) * exchange_rate).as_u64(0)
                    }
                    RoundingDirection::Up => {
                        (Number::from(amount.value) * exchange_rate).as_u64_ceil(0)
                    }
                },
            },
        };

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
        if (amount.notes == 0 && amount.tokens > 0) || (amount.tokens == 0 && amount.notes > 0) {
            return err!(crate::ErrorCode::InvalidAmount);
        }

        Ok(amount)
    }

    /// Get the exchange rate for deposit note -> token
    fn deposit_note_exchange_rate(&self) -> Number {
        let deposit_notes = std::cmp::max(1, self.deposit_notes);
        let total_value = std::cmp::max(Number::ONE, self.total_value());
        (total_value - *self.total_uncollected_fees()) / Number::from(deposit_notes)
    }

    /// Get the exchange rate for loan note -> token
    fn loan_note_exchange_rate(&self) -> Number {
        let loan_notes = std::cmp::max(1, self.loan_notes);
        let total_borrowed = std::cmp::max(Number::ONE, *self.total_borrowed());
        total_borrowed / Number::from(loan_notes)
    }

    /// Gets the total value of assets owned by/owed to the pool.
    fn total_value(&self) -> Number {
        *self.total_borrowed() + Number::from(self.deposit_tokens)
    }

    fn total_uncollected_fees_mut(&mut self) -> &mut Number {
        bytemuck::from_bytes_mut(&mut self.uncollected_fees)
    }

    fn total_uncollected_fees(&self) -> &Number {
        bytemuck::from_bytes(&self.uncollected_fees)
    }

    fn total_borrowed_mut(&mut self) -> &mut Number {
        bytemuck::from_bytes_mut(&mut self.borrowed_tokens)
    }

    fn total_borrowed(&self) -> &Number {
        bytemuck::from_bytes(&self.borrowed_tokens)
    }

    fn flags(&self) -> PoolFlags {
        PoolFlags::from_bits_truncate(self.config.flags)
    }
}

#[derive(Debug)]
pub struct FullAmount {
    pub tokens: u64,
    pub notes: u64,
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
    /// The exchange rate increases over time due to interest.
    /// The rate is notes:tokens, such that 1.2 means that 1 note = 1.2 tokens.
    /// This is because a user deposits 1 token and gets 1 note back (assuming 1:1 rate),
    /// they then earn interest due passage of time, and become entitled to
    /// 1.2 tokens, where 0.2 is the interest. Thus 1 note becomes 1.2 tokens.
    ///
    /// In an exchange where a user supplies notes, we multiply by the exchange rate
    /// to get tokens.
    /// In an exchange where a user supplies tokens, we divide by the exchange rate
    /// to get notes.
    ///
    /// `amount` can either be tokens or notes. The amount type (1), side of the position
    /// in the pool (2), and the instruction type (3), impact the rounding direction.
    /// We always want a rounding position that is favourable to the pool.
    /// The combination of the 3 factors is shown in the table below.
    ///
    /// | Instruction | Note Action     | Direction      | Rounding |
    /// | :---        |     :----:      |     :----:     |     ---: |
    /// | Deposit     | Mint Collateral | Tokens > Notes | Down     |
    /// | Deposit     | Mint Collateral | Notes > Tokens | Up       |
    /// | Withdraw    | Burn Collateral | Tokens > Notes | Up       |
    /// | Withdraw    | Burn Collateral | Notes > Tokens | Down     |
    /// | Borrow      | Mint Claim      | Tokens > Notes | Up       |
    /// | Borrow      | Mint Claim      | Notes > Tokens | Down     |
    /// | Repay       | Burn Claim      | Tokens > Notes | Down     |
    /// | Repay       | Burn Claim      | Notes > Tokens | Up       |
    pub const fn direction(pool_action: PoolAction, amount_kind: AmountKind) -> Self {
        use RoundingDirection::*;
        match (pool_action, amount_kind) {
            (PoolAction::Borrow, AmountKind::Tokens)
            | (PoolAction::Deposit, AmountKind::Notes)
            | (PoolAction::Repay, AmountKind::Notes)
            | (PoolAction::Withdraw, AmountKind::Tokens) => Up,
            (PoolAction::Borrow, AmountKind::Notes)
            | (PoolAction::Deposit, AmountKind::Tokens)
            | (PoolAction::Repay, AmountKind::Tokens)
            | (PoolAction::Withdraw, AmountKind::Notes) => Down,
        }
    }
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
#[derive(Debug, Default, AnchorDeserialize, AnchorSerialize, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_ser_tokens, Token};

    #[test]
    fn test_deposit_note_rounding() -> Result<()> {
        let mut margin_pool = MarginPool::default();

        margin_pool.deposit(&FullAmount {
            tokens: 1_000_000,
            notes: 900_000,
        });

        // Deposit note exchange rate is 1.111111_.
        // If a user withdraws 9 notes, they should get 9 or 10 tokens back
        // depending on the rounding.

        assert_eq!(
            margin_pool.deposit_note_exchange_rate().as_u64(-9),
            1111111111
        );

        let deposit_amount =
            margin_pool.convert_deposit_amount(Amount::notes(12), RoundingDirection::Down)?;

        assert_eq!(deposit_amount.notes, 12);
        assert_eq!(deposit_amount.tokens, 13); // ref [0]

        let deposit_amount =
            margin_pool.convert_deposit_amount(Amount::notes(18), RoundingDirection::Down)?;

        assert_eq!(deposit_amount.notes, 18);
        assert_eq!(deposit_amount.tokens, 19);

        let deposit_amount =
            margin_pool.convert_deposit_amount(Amount::notes(12), RoundingDirection::Up)?;

        assert_eq!(deposit_amount.notes, 12);
        assert_eq!(deposit_amount.tokens, 14); // ref [1]

        // A user requesting 1 note should never get 0 tokens back,
        // or 1 token should never get 0 notes back

        let deposit_amount =
            margin_pool.convert_deposit_amount(Amount::notes(1), RoundingDirection::Down)?;

        // When depositing, 1:1 would be advantageous to the user
        assert_eq!(deposit_amount.notes, 1);
        assert_eq!(deposit_amount.tokens, 1);

        let deposit_amount =
            margin_pool.convert_deposit_amount(Amount::notes(1), RoundingDirection::Up)?;

        // Depositing 2 tokens for 1 note is disadvantageous to the user
        // and protects the protocol's average exchange rate
        assert_eq!(deposit_amount.notes, 1);
        assert_eq!(deposit_amount.tokens, 2);

        // Check the default rounding for depositing notes, as it is disadvantageous
        // to the user per the previous observation.
        let direction = RoundingDirection::direction(PoolAction::Deposit, AmountKind::Notes);
        assert_eq!(RoundingDirection::Up, direction);

        // A repay is the same as a deposit (inflow)
        let direction = RoundingDirection::direction(PoolAction::Repay, AmountKind::Notes);
        assert_eq!(RoundingDirection::Up, direction);

        Ok(())
    }

    /// Conversion between tokens and notes would allow a user to
    /// provide tokens for notes, or to specify the number of tokens
    /// to receive on withdrawal.
    ///
    /// As the exchange rate between notes and tokens is expected to
    /// increase over time, there is a risk that a user could extract
    /// 1 token while burning 0 notes due to rounding.
    #[test]
    fn test_deposit_token_rounding() -> Result<()> {
        let mut margin_pool = MarginPool::default();

        margin_pool.deposit(&FullAmount {
            tokens: 1_000_000,
            notes: 900_000,
        });

        assert_eq!(
            margin_pool.deposit_note_exchange_rate().as_u64(-9),
            1111111111
        );

        let deposit_result =
            margin_pool.convert_deposit_amount(Amount::tokens(1), RoundingDirection::Down);

        // Rounding down would return 0 notes, which is invalid. This should error out
        assert!(deposit_result.is_err());

        let deposit_amount =
            margin_pool.convert_deposit_amount(Amount::tokens(1), RoundingDirection::Up)?;

        // Depositing 1 token for 1 note is disadvantageous to the user as they
        // get a lower rate than the 1.111_.
        // This is however because they are requesting the smallest unit, so
        // this test hides the true intention of the rounding.
        assert_eq!(deposit_amount.notes, 1);
        assert_eq!(deposit_amount.tokens, 1);

        // It is better observed with a bigger number.
        // The expectation when a user deposits is that they should get less notes
        // than the exchange rate if we have to round. This is because fewer notes
        // entitle the user to fewer tokens on withdrawal from the pool.

        // We start by rounding up a bigger number. See [0]
        let deposit_amount =
            margin_pool.convert_deposit_amount(Amount::tokens(9), RoundingDirection::Up)?;

        assert_eq!(deposit_amount.notes, 9);
        assert_eq!(deposit_amount.tokens, 9);

        // [1] shows the behaviour when rounding 12 notes up, we get 13 tokens.
        let deposit_amount =
            margin_pool.convert_deposit_amount(Amount::tokens(13), RoundingDirection::Up)?;

        assert_eq!(deposit_amount.tokens, 13);
        // [1] returned 12 notes, and we get 12 notes back.
        assert_eq!(deposit_amount.notes, 12);

        // If we round down instead of up, we preserve value.
        let deposit_amount =
            margin_pool.convert_deposit_amount(Amount::tokens(14), RoundingDirection::Down)?;

        assert_eq!(deposit_amount.tokens, 14);
        assert_eq!(deposit_amount.notes, 12);

        // From the above scenarios, we achieve a roundtrip when we change the
        // rounding direction depending on the conversion direction.
        // When depositing notes, we rounded up. When depositing tokens, rounding
        // down leaves the user in a comparable scenario.

        // Thus when depositing tokens, we should round down.
        let direction = RoundingDirection::direction(PoolAction::Deposit, AmountKind::Tokens);
        assert_eq!(RoundingDirection::Down, direction);

        // Repay should behave like deposit
        let direction = RoundingDirection::direction(PoolAction::Repay, AmountKind::Tokens);
        assert_eq!(RoundingDirection::Down, direction);

        Ok(())
    }

    #[test]
    fn test_loan_note_rounding() -> Result<()> {
        let mut margin_pool = MarginPool::default();
        margin_pool.config.flags = PoolFlags::ALLOW_LENDING.bits();

        // Deposit funds so there is liquidity
        margin_pool.deposit(&FullAmount {
            tokens: 1_000_000,
            notes: 1_000_000,
        });

        margin_pool.borrow(&FullAmount {
            tokens: 1_000_000,
            notes: 900_000,
        })?;

        assert_eq!(margin_pool.loan_note_exchange_rate().as_u64(-9), 1111111111);

        let loan_amount =
            margin_pool.convert_loan_amount(Amount::notes(1), RoundingDirection::Down)?;

        assert_eq!(loan_amount.notes, 1);
        assert_eq!(loan_amount.tokens, 1);

        let loan_amount =
            margin_pool.convert_loan_amount(Amount::notes(1), RoundingDirection::Up)?;

        // When withdrawing, rounding up benefits the user at the cost of the
        // protocol. The user gets to borrow at a lower rate (0.5 vs 1.111_).
        assert_eq!(loan_amount.notes, 1);
        assert_eq!(loan_amount.tokens, 2);

        // Check that borrow rounding is down, so the user does not borrow at
        // a lower rate.
        let direction = RoundingDirection::direction(PoolAction::Withdraw, AmountKind::Notes);
        assert_eq!(RoundingDirection::Down, direction);

        // A borrow is the same as withdraw (outflow)
        let direction = RoundingDirection::direction(PoolAction::Borrow, AmountKind::Notes);
        assert_eq!(RoundingDirection::Down, direction);

        Ok(())
    }

    #[test]
    fn test_loan_token_rounding() -> Result<()> {
        let mut margin_pool = MarginPool::default();
        margin_pool.config.flags = PoolFlags::ALLOW_LENDING.bits();

        margin_pool.deposit(&FullAmount {
            tokens: 1_000_000,
            notes: 1_000_000,
        });

        margin_pool.borrow(&FullAmount {
            tokens: 1_000_000,
            notes: 900_000,
        })?;

        assert_eq!(margin_pool.loan_note_exchange_rate().as_u64(-9), 1111111111);

        let loan_result =
            margin_pool.convert_loan_amount(Amount::tokens(1), RoundingDirection::Down);

        // Rounding down to 0 is not allowed
        assert!(loan_result.is_err());

        let loan_amount =
            margin_pool.convert_loan_amount(Amount::tokens(1), RoundingDirection::Up)?;

        // When withdrawing tokens, the user should get 111 tokens for 100 notes (or less)
        // at the current exchange rate. A 1:1 is disadvantageous to the user
        // as the user can borrow 111 times, and get 111 tokens for 111 notes,
        // which if they borrowed at once, they could have received more tokens.
        assert_eq!(loan_amount.notes, 1);
        assert_eq!(loan_amount.tokens, 1);

        let loan_amount =
            margin_pool.convert_loan_amount(Amount::tokens(111), RoundingDirection::Up)?;

        assert_eq!(loan_amount.tokens, 111);
        // Even at a larger quantity, rounding up is still disadvantageous as
        // the user borrows at a lower rate than the prevailing exchange rate.
        assert_eq!(loan_amount.notes, 100);

        // In this instance, there is a difference in rationale between borrowing
        // and withdrawing.
        // When borrowing, we mint loan notes, and would want to mint more notes
        // for the same tokens if rounding is involved.
        let direction = RoundingDirection::direction(PoolAction::Borrow, AmountKind::Tokens);
        assert_eq!(RoundingDirection::Up, direction);

        // When withdrawing from a deposit pool, we want to give the user
        // less tokens for more notes.
        // Thus the rounding in a withdrawal from tokens should be up,
        // as 1 token would mean more notes.
        let direction = RoundingDirection::direction(PoolAction::Withdraw, AmountKind::Tokens);
        assert_eq!(RoundingDirection::Up, direction);

        Ok(())
    }

    #[test]
    fn margin_pool_serialization() {
        let pool = MarginPool::default();
        assert_ser_tokens(
            &pool,
            &[
                Token::Struct {
                    name: "MarginPool",
                    len: 13,
                },
                Token::Str("version"),
                Token::U8(0),
                Token::Str("vault"),
                Token::Str("11111111111111111111111111111111"),
                Token::Str("feeDestination"),
                Token::Str("11111111111111111111111111111111"),
                Token::Str("depositNoteMint"),
                Token::Str("11111111111111111111111111111111"),
                Token::Str("loanNoteMint"),
                Token::Str("11111111111111111111111111111111"),
                Token::Str("tokenMint"),
                Token::Str("11111111111111111111111111111111"),
                Token::Str("tokenPriceOracle"),
                Token::Str("11111111111111111111111111111111"),
                Token::Str("borrowedTokens"),
                Token::Str("0.0"),
                Token::Str("uncollectedFees"),
                Token::Str("0.0"),
                Token::Str("depositTokens"),
                Token::U64(0),
                Token::Str("depositNotes"),
                Token::U64(0),
                Token::Str("loanNotes"),
                Token::U64(0),
                Token::Str("accruedUntil"),
                Token::I64(0),
                Token::StructEnd,
            ],
        );
    }
}
