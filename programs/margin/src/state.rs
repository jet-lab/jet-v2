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

use anchor_lang::{prelude::*, system_program, Discriminator};
use bytemuck::{Contiguous, Pod, Zeroable};
use jet_metadata::TokenKind;
#[cfg(any(test, feature = "cli"))]
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::{
    util::Require, ErrorCode, PriceChangeInfo, MAX_ORACLE_CONFIDENCE, MAX_ORACLE_STALENESS,
    MAX_PRICE_QUOTE_AGE, MIN_COLLATERAL_RATIO,
};
use jet_proto_math::Number128;
use jet_proto_proc_macros::assert_size;

use anchor_lang::Result as AnchorResult;
use std::{convert::TryFrom, result::Result};

const POS_PRICE_VALID: u8 = 1;

#[account(zero_copy)]
#[repr(C)]
// bytemuck requires a higher alignment than 1 for unit tests to run.
#[cfg_attr(not(target_arch = "bpf"), repr(align(8)))]
pub struct MarginAccount {
    pub version: u8,
    pub bump_seed: [u8; 1],
    pub user_seed: [u8; 2],

    pub reserved0: [u8; 4],

    /// The owner of this account, which generally has to sign for any changes to it
    pub owner: Pubkey,

    /// The state of an active liquidation for this account
    pub liquidation: Pubkey,

    /// The active liquidator for this account
    pub liquidator: Pubkey,

    /// The storage for tracking account balances
    pub positions: [u8; 7432],
}

#[cfg(any(test, feature = "cli"))]
impl Serialize for MarginAccount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("MarginAccount", 5)?;
        s.serialize_field("version", &self.version)?;
        s.serialize_field("owner", &self.owner.to_string())?;
        s.serialize_field("liquidation", &self.liquidation.to_string())?;
        s.serialize_field("liquidator", &self.liquidator.to_string())?;
        s.serialize_field("positions", &self.positions().collect::<Vec<_>>())?;
        s.end()
    }
}

impl std::fmt::Debug for MarginAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut acc = f.debug_struct("MarginAccount");
        acc.field("version", &self.version)
            .field("bump_seed", &self.bump_seed)
            .field("user_seed", &self.user_seed)
            .field("reserved0", &self.reserved0)
            .field("owner", &self.owner)
            .field("liquidation", &self.liquidation)
            .field("liquidator", &self.liquidator);

        if self.positions().next().is_some() {
            acc.field("positions", &self.positions().collect::<Vec<_>>());
        } else {
            acc.field("positions", &Vec::<AccountPosition>::new());
        }

        acc.finish()
    }
}

/// Execute all the mandatory anchor account verifications that are used during deserialization
/// - performance: don't have to deserialize (even zero_copy copies)
/// - compatibility: straightforward validation for programs using different anchor versions and non-anchor programs
pub trait AnchorVerify: Discriminator + Owner {
    fn anchor_verify(info: &AccountInfo) -> AnchorResult<()> {
        if info.owner == &system_program::ID && info.lamports() == 0 {
            return err!(anchor_lang::error::ErrorCode::AccountNotInitialized);
        }
        if info.owner != &Self::owner() {
            return Err(
                Error::from(anchor_lang::error::ErrorCode::AccountOwnedByWrongProgram)
                    .with_pubkeys((*info.owner, MarginAccount::owner())),
            );
        }
        let data: &[u8] = &info.try_borrow_data()?;
        if data.len() < Self::discriminator().len() {
            return Err(anchor_lang::error::ErrorCode::AccountDiscriminatorNotFound.into());
        }
        let given_disc = &data[..8];
        if Self::discriminator() != given_disc {
            return Err(anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch.into());
        }
        Ok(())
    }
}

impl AnchorVerify for MarginAccount {}

impl MarginAccount {
    pub fn start_liquidation(&mut self, liquidation: Pubkey, liquidator: Pubkey) {
        self.liquidation = liquidation;
        self.liquidator = liquidator;
    }

    pub fn end_liquidation(&mut self) {
        self.liquidation = Pubkey::default();
        self.liquidator = Pubkey::default();
    }

    pub fn verify_not_liquidating(&self) -> AnchorResult<()> {
        if self.liquidation != Pubkey::default() {
            msg!("account is being liquidated");
            Err(ErrorCode::Liquidating.into())
        } else {
            Ok(())
        }
    }

    pub fn initialize(&mut self, owner: Pubkey, seed: u16, bump_seed: u8) {
        self.owner = owner;
        self.bump_seed = [bump_seed];
        self.user_seed = seed.to_le_bytes();
        self.liquidator = Pubkey::default();
    }

    /// Get the list of positions on this account
    pub fn positions(&self) -> impl Iterator<Item = &AccountPosition> {
        self.position_list()
            .positions
            .iter()
            .filter(|p| p.address != Pubkey::default())
    }

    /// Register the space for a new position into this account
    #[allow(clippy::too_many_arguments)]
    pub fn register_position(
        &mut self,
        token: Pubkey,
        decimals: u8,
        address: Pubkey,
        adapter: Pubkey,
        kind: PositionKind,
        collateral_weight: u16,
        collateral_max_staleness: u64,
    ) -> AnchorResult<()> {
        let free_position = self.position_list_mut().add(token)?;

        free_position.exponent = -(decimals as i16);
        free_position.address = address;
        free_position.adapter = adapter;
        free_position.kind = kind.into_integer();
        free_position.balance = 0;
        free_position.collateral_weight = collateral_weight;
        free_position.collateral_max_staleness = collateral_max_staleness;

        Ok(())
    }

    /// Free the space from a previously registered position no longer needed
    pub fn unregister_position(&mut self, mint: &Pubkey, account: &Pubkey) -> AnchorResult<()> {
        let removed = self.position_list_mut().remove(mint, account)?;

        if removed.balance != 0 {
            return err!(ErrorCode::CloseNonZeroPosition);
        }

        if removed.flags.contains(AdapterPositionFlags::REQUIRED) {
            return err!(ErrorCode::CloseRequiredPosition);
        }

        Ok(())
    }

    pub fn get_position_mut(&mut self, mint: &Pubkey) -> Option<&mut AccountPosition> {
        self.position_list_mut().get_mut(mint)
    }

    /// Change the balance for a position
    pub fn set_position_balance(
        &mut self,
        mint: &Pubkey,
        account: &Pubkey,
        balance: u64,
    ) -> Result<(), ErrorCode> {
        let position = self.position_list_mut().get_mut(mint).require()?;

        if position.address != *account {
            return Err(ErrorCode::PositionNotRegistered);
        }

        position.set_balance(balance);
        Ok(())
    }

    /// Change the current price value of a position
    pub fn set_position_price(
        &mut self,
        mint: &Pubkey,
        price: &PriceInfo,
    ) -> Result<(), ErrorCode> {
        self.position_list_mut()
            .get_mut(mint)
            .require()?
            .set_price(price)
    }

    /// Check that the overall health of the account is acceptable, by comparing the
    /// total value of the claims versus the available collateral. If the collateralization
    /// ratio is above the minimum, then the account is considered healthy.
    pub fn verify_healthy_positions(&self) -> AnchorResult<()> {
        let info = self.valuation()?;
        let min_ratio = Number128::from_bps(MIN_COLLATERAL_RATIO);

        match info.c_ratio() {
            Some(c_ratio) if c_ratio < min_ratio => {
                msg!("Account unhealty. C-ratio: {}", c_ratio.to_string());
                err!(ErrorCode::Unhealthy)
            }
            _ if info.past_due() => {
                msg!("Account unhealty. Debt is past due");
                err!(ErrorCode::Unhealthy)
            }
            _ => Ok(()),
        }
    }

    /// Check that the overall health of the account is *not* acceptable.
    pub fn verify_unhealthy_positions(&self) -> AnchorResult<()> {
        let info = self.valuation()?;
        let min_ratio = Number128::from_bps(MIN_COLLATERAL_RATIO);

        if info.stale_collateral > Number128::ZERO {
            for (position_token, error) in info.stale_collateral_list {
                msg!("stale position {}: {}", position_token, error)
            }
            return Err(error!(ErrorCode::StalePositions));
        }

        match info.c_ratio() {
            Some(c_ratio) if c_ratio < min_ratio => Ok(()),
            _ if info.past_due() => Ok(()),
            _ => Err(error!(ErrorCode::Healthy)),
        }
    }

    /// Check if the given address is an authority for this margin account
    pub fn has_authority(&self, authority: Pubkey) -> bool {
        authority == self.owner || authority == self.liquidator
    }

    pub fn valuation(&self) -> AnchorResult<Valuation> {
        let timestamp = crate::util::get_timestamp();

        let mut fresh_collateral = Number128::ZERO;
        let mut stale_collateral = Number128::ZERO;
        let mut claims = Number128::ZERO;
        let mut past_due = false;

        let mut stale_collateral_list = vec![];

        for position in self.positions() {
            let kind = PositionKind::from_integer(position.kind).unwrap();
            let stale_reason = {
                let balance_age = timestamp - position.balance_timestamp;
                let price_quote_age = timestamp - position.price.timestamp;

                if position.price.is_valid != POS_PRICE_VALID {
                    // collateral with bad prices
                    Some(ErrorCode::InvalidPrice)
                } else if position.collateral_max_staleness > 0
                    && balance_age > position.collateral_max_staleness
                {
                    // outdated balance
                    Some(ErrorCode::OutdatedBalance)
                } else if price_quote_age > MAX_PRICE_QUOTE_AGE {
                    // outdated price
                    Some(ErrorCode::OutdatedPrice)
                } else {
                    None
                }
            };

            //TODO JV2M-360
            //TODO user replays a loan but Claim still has a position.value()
            match (kind, stale_reason) {
                (PositionKind::NoValue, _) => (),
                (PositionKind::Claim, None) => {
                    if position.balance > 0
                        && position.flags.contains(AdapterPositionFlags::PAST_DUE)
                    {
                        past_due = true;
                    }
                    claims += position.value()
                }
                (PositionKind::Claim, Some(error)) => return Err(error!(error)),

                (PositionKind::Deposit, None) => fresh_collateral += position.collateral_value(),
                (PositionKind::Deposit, Some(e)) => {
                    stale_collateral += position.collateral_value();
                    stale_collateral_list.push((position.token, e));
                }
            }
        }

        Ok(Valuation {
            fresh_collateral,
            stale_collateral,
            stale_collateral_list,
            claims,
            past_due,
        })
    }

    fn position_list(&self) -> &AccountPositionList {
        bytemuck::from_bytes(&self.positions)
    }

    fn position_list_mut(&mut self) -> &mut AccountPositionList {
        bytemuck::from_bytes_mut(&mut self.positions)
    }
}

pub trait SignerSeeds<const SIZE: usize> {
    fn signer_seeds(&self) -> [&[u8]; SIZE];
    fn signer_seeds_owned(&self) -> Box<dyn SignerSeeds<SIZE>>;
}

impl<const A: usize, const B: usize, const C: usize> SignerSeeds<3>
    for ([u8; A], [u8; B], [u8; C])
{
    fn signer_seeds(&self) -> [&[u8]; 3] {
        let (s0, s1, s2) = self;
        [s0, s1, s2]
    }

    fn signer_seeds_owned(&self) -> Box<dyn SignerSeeds<3>> {
        Box::new(*self)
    }
}

impl SignerSeeds<3> for MarginAccount {
    fn signer_seeds(&self) -> [&[u8]; 3] {
        [
            self.owner.as_ref(),
            self.user_seed.as_ref(),
            self.bump_seed.as_ref(),
        ]
    }

    fn signer_seeds_owned(&self) -> Box<dyn SignerSeeds<3>> {
        Box::new((self.owner.to_bytes(), self.user_seed, self.bump_seed))
    }
}

#[assert_size(24)]
#[derive(
    Pod, Zeroable, AnchorSerialize, AnchorDeserialize, Debug, Default, Clone, Copy, Eq, PartialEq,
)]
#[cfg_attr(
    any(test, feature = "cli"),
    derive(serde::Serialize),
    serde(rename_all = "camelCase")
)]
#[repr(C)]
pub struct PriceInfo {
    /// The current price
    pub value: i64,

    /// The timestamp the price was valid at
    pub timestamp: u64,

    /// The exponent for the price value
    pub exponent: i32,

    /// Flag indicating if the price is valid for the position
    pub is_valid: u8,

    #[cfg_attr(any(test, feature = "cli"), serde(skip_serializing))]
    pub _reserved: [u8; 3],
}

impl PriceInfo {
    pub fn new_valid(exponent: i32, value: i64, timestamp: u64) -> Self {
        Self {
            value,
            exponent,
            timestamp,
            is_valid: POS_PRICE_VALID,
            _reserved: [0u8; 3],
        }
    }

    pub fn new_invalid() -> Self {
        Self {
            value: 0,
            exponent: 0,
            timestamp: 0,
            is_valid: 0,
            _reserved: [0u8; 3],
        }
    }
}

impl TryFrom<PriceChangeInfo> for PriceInfo {
    type Error = anchor_lang::error::Error;

    fn try_from(value: PriceChangeInfo) -> AnchorResult<Self> {
        let clock = Clock::get()?;
        let max_confidence = Number128::from_bps(MAX_ORACLE_CONFIDENCE);

        let twap = Number128::from_decimal(value.twap, value.exponent);
        let confidence = Number128::from_decimal(value.confidence, value.exponent);

        let price = match (confidence, value.publish_time) {
            (c, _) if (c / twap) > max_confidence => PriceInfo::new_invalid(),
            (_, publish_time) if (clock.unix_timestamp - publish_time) > MAX_ORACLE_STALENESS => {
                PriceInfo::new_invalid()
            }
            _ => PriceInfo::new_valid(value.exponent, value.value, clock.unix_timestamp as u64),
        };

        Ok(price)
    }
}

#[derive(Debug, Clone, Copy, Contiguous, Eq, PartialEq)]
#[repr(u32)]
pub enum PositionKind {
    /// The position is not worth anything
    NoValue,

    /// The position contains a balance of available collateral
    Deposit,

    /// The position contains a balance of tokens that are owed as a part of some debt.
    Claim,
}

impl From<TokenKind> for PositionKind {
    fn from(token: TokenKind) -> Self {
        match token {
            TokenKind::NonCollateral => PositionKind::NoValue,
            TokenKind::Collateral => PositionKind::Deposit,
            TokenKind::Claim => PositionKind::Claim,
        }
    }
}

#[assert_size(192)]
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Copy)]
#[repr(C)]
pub struct AccountPosition {
    /// The address of the token/mint of the asset
    pub token: Pubkey,

    /// The address of the account holding the tokens.
    pub address: Pubkey,

    /// The address of the adapter managing the asset
    pub adapter: Pubkey,

    /// The current value of this position, stored as a `Number128` with fixed precision.
    pub value: [u8; 16],

    /// The amount of tokens in the account
    pub balance: u64,

    /// The timestamp of the last balance update
    pub balance_timestamp: u64,

    /// The current price/value of each token
    pub price: PriceInfo,

    /// The kind of balance this position contains
    pub kind: u32,

    /// The exponent for the token value
    pub exponent: i16,

    /// A weight on the value of this asset when counting collateral
    pub collateral_weight: u16,

    /// The max staleness for the account balance (seconds)
    pub collateral_max_staleness: u64,

    /// Flags that are set by the adapter
    pub flags: AdapterPositionFlags,

    _reserved: [u8; 23],
}

bitflags::bitflags! {
    #[derive(AnchorSerialize, AnchorDeserialize, Default)]
    pub struct AdapterPositionFlags: u8 {
        /// The position may never be removed by the user, even if the balance remains at zero,
        /// until the adapter explicitly unsets this flag.
        const REQUIRED = 1 << 0;

        /// Only applies to claims
        /// For any other position, this can be set, but it will be ignored.
        /// The claim must be repaid immediately.
        /// The account will be considered unhealty if there is any balance on this position.
        const PAST_DUE = 1 << 1;
    }
}

impl AccountPosition {
    pub fn calculate_value(&mut self) {
        self.value = (Number128::from_decimal(self.balance, self.exponent)
            * Number128::from_decimal(self.price.value, self.price.exponent))
        .into_bits();
    }

    pub fn value(&self) -> Number128 {
        Number128::from_bits(self.value)
    }

    pub fn collateral_value(&self) -> Number128 {
        Number128::from_bps(self.collateral_weight) * self.value()
    }

    /// Update the balance for this position
    fn set_balance(&mut self, balance: u64) {
        self.balance = balance;
        self.balance_timestamp = crate::util::get_timestamp();
        self.calculate_value();
    }

    /// Update the price for this position
    pub fn set_price(&mut self, price: &PriceInfo) -> Result<(), ErrorCode> {
        self.price = *price;
        self.calculate_value();

        Ok(())
    }
}

impl std::fmt::Debug for AccountPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut acc = f.debug_struct("AccountPosition");
        acc.field("token", &self.token)
            .field("address", &self.address)
            .field("adapter", &self.adapter)
            .field("value", &self.value().to_string())
            .field("balance", &self.balance)
            .field("balance_timestamp", &self.balance_timestamp)
            .field("price", &self.price)
            .field("kind", &self.kind)
            .field("exponent", &self.exponent)
            .field("collateral_weight", &self.collateral_weight)
            .field("collateral_max_staleness", &self.collateral_max_staleness);

        acc.finish()
    }
}

#[cfg(any(test, feature = "cli"))]
impl Serialize for AccountPosition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("AccountPosition", 11)?;
        s.serialize_field("address", &self.address.to_string())?;
        s.serialize_field("token", &self.token.to_string())?;
        s.serialize_field("adapter", &self.adapter.to_string())?;
        s.serialize_field("value", &self.value().to_string())?;
        s.serialize_field("balance", &self.balance)?;
        s.serialize_field("balanceTimestamp", &self.balance_timestamp)?;
        s.serialize_field("price", &self.price)?;
        s.serialize_field("kind", &self.kind)?;
        s.serialize_field("exponent", &self.exponent)?;
        s.serialize_field("collateralWeight", &self.collateral_weight)?;
        s.serialize_field("collateralMaxStaleness", &self.collateral_max_staleness)?;
        s.end()
    }
}

#[assert_size(40)]
#[derive(AnchorSerialize, AnchorDeserialize, Default, Pod, Zeroable, Debug, Clone, Copy)]
#[repr(C)]
pub struct AccountPositionKey {
    /// The address of the mint for the position token
    mint: Pubkey,

    /// The array index where the data for this position is located
    index: usize,
}

///
#[assert_size(7432)]
#[derive(AnchorSerialize, AnchorDeserialize, Default, Pod, Zeroable, Debug, Clone, Copy)]
#[repr(C)]
pub struct AccountPositionList {
    pub length: usize,
    pub map: [AccountPositionKey; 32],
    pub positions: [AccountPosition; 32],
}

impl AccountPositionList {
    /// Add a position to the position list.
    ///
    /// Finds an empty slot in `map` and `positions`, and adds an empty position
    /// to the slot.
    pub fn add(&mut self, mint: Pubkey) -> AnchorResult<&mut AccountPosition> {
        // verify there's no existing position
        if self.map.iter().any(|p| p.mint == mint) {
            return err!(ErrorCode::PositionAlreadyRegistered);
        }

        // find the first free space to store the position info
        let (index, free_position) = self
            .positions
            .iter_mut()
            .enumerate()
            .find(|(_, p)| p.token == Pubkey::default())
            .ok_or_else(|| error!(ErrorCode::MaxPositions))?;

        // add the new entry to the sorted map
        self.map[self.length] = AccountPositionKey { mint, index };

        self.length += 1;
        (&mut self.map[..self.length]).sort_by_key(|p| p.mint);

        // mark position as not free
        free_position.token = mint;

        // return the allocated position to be initialized further
        Ok(free_position)
    }

    /// Remove a position from the margin account.
    ///
    /// # Error
    ///
    /// - If an account with the `mint` does not exist.
    /// - If the position's address is not the same as the `account`
    pub fn remove(&mut self, mint: &Pubkey, account: &Pubkey) -> AnchorResult<AccountPosition> {
        let map_index = self.get_map_index(mint).require()?;
        // Get the map whose position to remove
        let map = self.map[map_index];
        // Take a copy of the position to be removed
        let position = self.positions[map.index];
        // Check that the position is correct
        if &position.address != account {
            return err!(ErrorCode::PositionNotRegistered);
        }
        // Remove the position
        let freed_position = bytemuck::bytes_of_mut(&mut self.positions[map.index]);
        freed_position.fill(0);

        // Move the map elements up by 1 to replace map position being removed
        (&mut self.map).copy_within(map_index + 1..self.length, map_index);

        self.length -= 1;
        // Clear the map at the last slot of the array, as it is shifted up
        self.map[self.length].mint = Pubkey::default();
        self.map[self.length].index = 0;

        Ok(position)
    }

    pub fn get(&self, mint: &Pubkey) -> Option<&AccountPosition> {
        let key = self.get_key(mint)?;
        let position = &self.positions[key.index];

        Some(position)
    }

    pub fn get_mut(&mut self, mint: &Pubkey) -> Option<&mut AccountPosition> {
        let key = self.get_key(mint)?;
        let position = &mut self.positions[key.index];

        Some(position)
    }

    fn get_key(&self, mint: &Pubkey) -> Option<&AccountPositionKey> {
        Some(&self.map[self.get_map_index(mint)?])
    }

    fn get_map_index(&self, mint: &Pubkey) -> Option<usize> {
        (&self.map[..self.length])
            .binary_search_by_key(mint, |p| p.mint)
            .ok()
    }
}

unsafe impl Zeroable for AccountPosition {}
unsafe impl Pod for AccountPosition {}

/// State of an in-progress liquidation
#[account(zero_copy)]
#[repr(C, align(8))]
#[derive(Debug)]
pub struct Liquidation {
    /// time that liquidate_begin initialized this liquidation
    pub start_time: i64,

    /// cumulative change in value caused by invocations during the liquidation so far
    /// negative if value is lost
    pub value_change: Number128,

    /// cumulative change to c-ratio caused by invocations during the liquidation so far
    /// negative if c-ratio goes down
    pub c_ratio_change: Number128,

    /// lowest amount of value change that is allowed during invoke steps
    /// typically negative or zero
    /// if value_change goes lower than this number, liquidate_invoke should fail
    pub min_value_change: Number128,
}

impl Default for Liquidation {
    fn default() -> Self {
        Self {
            start_time: Default::default(),
            value_change: Number128::ZERO,
            c_ratio_change: Number128::ZERO,
            min_value_change: Number128::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Valuation {
    fresh_collateral: Number128,
    stale_collateral: Number128,
    stale_collateral_list: Vec<(Pubkey, ErrorCode)>,
    claims: Number128,
    past_due: bool,
}

impl Valuation {
    pub fn c_ratio(&self) -> Option<Number128> {
        if self.claims == Number128::ZERO {
            return None;
        }

        Some(self.fresh_collateral / self.claims)
    }

    pub fn net(&self) -> Number128 {
        self.fresh_collateral - self.claims
    }

    pub fn claims(&self) -> Number128 {
        self.claims
    }

    pub fn collateral(&self) -> Number128 {
        self.fresh_collateral
    }

    pub fn past_due(&self) -> bool {
        self.past_due
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jet_metadata::TokenKind;
    use serde_test::{assert_ser_tokens, Token};

    fn create_position_input(margin_address: &Pubkey) -> (Pubkey, Pubkey) {
        let token = Pubkey::new_unique();
        let (address, _) =
            Pubkey::find_program_address(&[margin_address.as_ref(), token.as_ref()], &crate::id());
        (token, address)
    }

    #[test]
    fn margin_account_debug() {
        let mut acc = MarginAccount {
            version: 1,
            bump_seed: [0],
            user_seed: [0; 2],
            reserved0: [0; 4],
            owner: Pubkey::default(),
            liquidation: Pubkey::default(),
            liquidator: Pubkey::default(),
            positions: [0; 7432],
        };
        let output = "MarginAccount { version: 1, bump_seed: [0], user_seed: [0, 0], reserved0: [0, 0, 0, 0], owner: 11111111111111111111111111111111, liquidation: 11111111111111111111111111111111, liquidator: 11111111111111111111111111111111, positions: [] }";
        assert_eq!(output, &format!("{:?}", acc));

        // use a non-default pubkey
        let key = crate::id();

        acc.register_position(key, 2, key, key, PositionKind::NoValue, 5000, 1000)
            .unwrap();
        let position = "AccountPosition { token: JPMRGNgRk3w2pzBM1RLNBnpGxQYsFQ3yXKpuk4tTXVZ, address: JPMRGNgRk3w2pzBM1RLNBnpGxQYsFQ3yXKpuk4tTXVZ, adapter: JPMRGNgRk3w2pzBM1RLNBnpGxQYsFQ3yXKpuk4tTXVZ, value: \"0.0\", balance: 0, balance_timestamp: 0, price: PriceInfo { value: 0, timestamp: 0, exponent: 0, is_valid: 0, _reserved: [0, 0, 0] }, kind: 0, exponent: -2, collateral_weight: 5000, collateral_max_staleness: 1000 }";
        let output = output.replace("positions: []", &format!("positions: [{}]", position));
        assert_eq!(&output, &format!("{:?}", acc));
    }

    #[test]
    fn margin_account_serialize() {
        let account = MarginAccount {
            version: 1,
            bump_seed: [0],
            user_seed: [0; 2],
            reserved0: [0; 4],
            owner: Pubkey::default(),
            liquidation: Pubkey::default(),
            liquidator: Pubkey::default(),
            positions: [0; 7432],
        };

        assert_ser_tokens(
            &account,
            &[
                Token::Struct {
                    name: "MarginAccount",
                    len: 5,
                },
                Token::Str("version"),
                Token::U8(1),
                Token::Str("owner"),
                Token::Str("11111111111111111111111111111111"),
                Token::Str("liquidation"),
                Token::Str("11111111111111111111111111111111"),
                Token::Str("liquidator"),
                Token::Str("11111111111111111111111111111111"),
                Token::Str("positions"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn account_position_serialize() {
        let position = AccountPosition {
            address: Pubkey::default(),
            token: Pubkey::default(),
            adapter: Pubkey::default(),
            value: [0u8; 16],
            balance: u64::default(),
            balance_timestamp: u64::default(),
            price: PriceInfo::default(),
            kind: u32::default(),
            exponent: i16::default(),
            collateral_weight: u16::default(),
            collateral_max_staleness: u64::default(),
            flags: AdapterPositionFlags::default(),
            _reserved: [0; 23],
        };

        assert_ser_tokens(
            &position,
            &[
                Token::Struct {
                    name: "AccountPosition",
                    len: 11,
                },
                Token::Str("address"),
                Token::Str("11111111111111111111111111111111"),
                Token::Str("token"),
                Token::Str("11111111111111111111111111111111"),
                Token::Str("adapter"),
                Token::Str("11111111111111111111111111111111"),
                Token::Str("value"),
                Token::Str("0.0"),
                Token::Str("balance"),
                Token::U64(0),
                Token::Str("balanceTimestamp"),
                Token::U64(0),
                Token::Str("price"),
                Token::Struct {
                    name: "PriceInfo",
                    len: 4,
                },
                Token::Str("value"),
                Token::I64(0),
                Token::Str("timestamp"),
                Token::U64(0),
                Token::Str("exponent"),
                Token::I32(0),
                Token::Str("isValid"),
                Token::U8(0),
                Token::StructEnd,
                Token::Str("kind"),
                Token::U32(0),
                Token::Str("exponent"),
                Token::I16(0),
                Token::Str("collateralWeight"),
                Token::U16(0),
                Token::Str("collateralMaxStaleness"),
                Token::U64(0),
                Token::StructEnd,
            ],
        )
    }

    #[test]
    fn test_mutate_positions() {
        let margin_address = Pubkey::new_unique();
        let adapter = Pubkey::new_unique();
        let mut margin_account = MarginAccount {
            version: 1,
            bump_seed: [0],
            user_seed: [0; 2],
            reserved0: [0; 4],
            owner: Pubkey::new_unique(),
            liquidation: Pubkey::default(),
            liquidator: Pubkey::default(),
            positions: [0; 7432],
        };

        // // Register a few positions, randomise the order
        let (token_e, address_e) = create_position_input(&margin_address);
        let (token_a, address_a) = create_position_input(&margin_address);
        let (token_d, address_d) = create_position_input(&margin_address);
        let (token_c, address_c) = create_position_input(&margin_address);
        let (token_b, address_b) = create_position_input(&margin_address);

        margin_account
            .register_position(token_a, 6, address_a, adapter, PositionKind::Deposit, 0, 0)
            .unwrap();

        margin_account
            .register_position(token_b, 6, address_b, adapter, PositionKind::Claim, 0, 0)
            .unwrap();

        margin_account
            .register_position(token_c, 6, address_c, adapter, PositionKind::Deposit, 0, 0)
            .unwrap();

        // Set and unset a position's balance
        margin_account
            .set_position_balance(&token_a, &address_a, 100)
            .unwrap();
        margin_account
            .set_position_balance(&token_a, &address_a, 0)
            .unwrap();

        // Unregister positions
        margin_account
            .unregister_position(&token_a, &address_a)
            .unwrap();
        assert_eq!(margin_account.positions().count(), 2);
        margin_account
            .unregister_position(&token_b, &address_b)
            .unwrap();
        assert_eq!(margin_account.positions().count(), 1);

        margin_account
            .register_position(
                token_e,
                9,
                address_e,
                adapter,
                PositionKind::NoValue,
                0,
                100,
            )
            .unwrap();
        assert_eq!(margin_account.positions().count(), 2);

        margin_account
            .register_position(
                token_d,
                9,
                address_d,
                adapter,
                PositionKind::NoValue,
                0,
                100,
            )
            .unwrap();
        assert_eq!(margin_account.positions().count(), 3);

        // It should not be possible to unregister mismatched token & position
        assert!(margin_account
            .unregister_position(&token_c, &address_b)
            .is_err());

        margin_account
            .unregister_position(&token_c, &address_c)
            .unwrap();
        margin_account
            .unregister_position(&token_e, &address_e)
            .unwrap();
        margin_account
            .unregister_position(&token_d, &address_d)
            .unwrap();

        // There should be no positions left
        assert_eq!(margin_account.positions().count(), 0);
        assert_eq!(margin_account.positions, [0; 7432]);
    }

    #[test]
    fn margin_account_past_due() {
        let mut acc = MarginAccount {
            version: 1,
            bump_seed: [0],
            user_seed: [0; 2],
            reserved0: [0; 4],
            owner: Pubkey::default(),
            liquidation: Pubkey::default(),
            liquidator: Pubkey::default(),
            positions: [0; 7432],
        };
        let collateral = register_position(&mut acc, 0, TokenKind::Collateral);
        let claim = register_position(&mut acc, 1, TokenKind::Claim);
        set_price(&mut acc, collateral, 100);
        set_price(&mut acc, claim, 100);
        acc.set_position_balance(&claim, &claim, 1).unwrap();
        assert_unhealthy(&acc);
        // show that this collateral is sufficient to cover the debt
        acc.set_position_balance(&collateral, &collateral, 100)
            .unwrap();
        assert_healthy(&acc);
        // but when past due, the account is unhealthy
        acc.get_position_mut(&claim).require().unwrap().flags |= AdapterPositionFlags::PAST_DUE;
        assert_unhealthy(&acc);
    }

    fn register_position(acc: &mut MarginAccount, index: u8, kind: TokenKind) -> Pubkey {
        let key = Pubkey::find_program_address(&[&[index]], &crate::id()).0;
        acc.register_position(key, 2, key, key, kind.into(), 10000, 0)
            .unwrap();

        key
    }

    fn assert_unhealthy(acc: &MarginAccount) {
        acc.verify_healthy_positions().unwrap_err();
        acc.verify_unhealthy_positions().unwrap();
    }

    fn assert_healthy(acc: &MarginAccount) {
        acc.verify_healthy_positions().unwrap();
        acc.verify_unhealthy_positions().unwrap_err();
    }

    fn set_price(acc: &mut MarginAccount, key: Pubkey, price: i64) {
        acc.set_position_price(
            &key,
            // &key,
            &PriceInfo {
                value: price,
                timestamp: crate::util::get_timestamp(),
                exponent: 1,
                is_valid: 1,
                _reserved: [0; 3],
            },
        )
        .unwrap()
    }

    #[test]
    fn proper_account_passes_anchor_verify() {
        MarginAccount::anchor_verify(&AccountInfo::new(
            &Pubkey::default(),
            true,
            true,
            &mut 0,
            &mut MarginAccount::discriminator(),
            &crate::id(),
            true,
            0,
        ))
        .unwrap();
    }

    #[test]
    fn wrong_owner_fails_anchor_verify() {
        MarginAccount::anchor_verify(&AccountInfo::new(
            &Pubkey::default(),
            true,
            true,
            &mut 0,
            &mut MarginAccount::discriminator(),
            &Pubkey::default(),
            true,
            0,
        ))
        .unwrap_err();
    }

    #[test]
    fn wrong_discriminator_fails_anchor_verify() {
        MarginAccount::anchor_verify(&AccountInfo::new(
            &Pubkey::default(),
            true,
            true,
            &mut 0,
            &mut [0, 1, 2, 3, 4, 5, 6, 7],
            &crate::id(),
            true,
            0,
        ))
        .unwrap_err();
    }

    #[test]
    fn no_data_fails_anchor_verify() {
        MarginAccount::anchor_verify(&AccountInfo::new(
            &Pubkey::default(),
            true,
            true,
            &mut 0,
            &mut [],
            &crate::id(),
            true,
            0,
        ))
        .unwrap_err();
    }
}
