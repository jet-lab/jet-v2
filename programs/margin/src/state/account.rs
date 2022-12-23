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
use std::convert::TryFrom;

#[cfg(any(test, feature = "cli"))]
use serde::ser::{Serialize, SerializeStruct, Serializer};

use jet_program_common::Number128;

use anchor_lang::Result as AnchorResult;
use std::result::Result;

use crate::{
    syscall::{sys, Sys},
    util::{Invocation, Require},
    ErrorCode, TokenKind, MAX_PRICE_QUOTE_AGE, MAX_USER_POSITIONS,
};

mod positions;

pub use positions::*;

/// The current version for the margin account state
pub const MARGIN_ACCOUNT_VERSION: u8 = 1;

#[account(zero_copy)]
#[repr(C)]
// bytemuck requires a higher alignment than 1 for unit tests to run.
#[cfg_attr(not(target_arch = "bpf"), repr(align(8)))]
pub struct MarginAccount {
    pub version: u8,
    pub bump_seed: [u8; 1],
    pub user_seed: [u8; 2],

    /// Data an adapter can use to check what the margin program thinks about the current invocation
    /// Must normally be zeroed, except during an invocation.
    pub invocation: Invocation,

    pub reserved0: [u8; 3],

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
            .field("invocation", &self.invocation)
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
        if self.is_liquidating() {
            msg!("account is being liquidated");
            Err(ErrorCode::Liquidating.into())
        } else {
            Ok(())
        }
    }

    pub fn is_liquidating(&self) -> bool {
        self.liquidation != Pubkey::default()
    }

    pub fn initialize(&mut self, owner: Pubkey, seed: u16, bump_seed: u8) {
        self.version = MARGIN_ACCOUNT_VERSION;
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
        kind: TokenKind,
        value_modifier: u16,
        max_staleness: u64,
        approvals: &[Approver],
    ) -> AnchorResult<AccountPositionKey> {
        if !self.is_liquidating() && self.position_list().length >= MAX_USER_POSITIONS {
            return err!(ErrorCode::MaxPositions);
        }

        let (key, free_position) = self.position_list_mut().add(token)?;

        free_position.exponent = -(decimals as i16);
        free_position.address = address;
        free_position.adapter = adapter;
        free_position.kind = kind.into_integer();
        free_position.balance = 0;
        free_position.value_modifier = value_modifier;
        free_position.max_staleness = max_staleness;

        if !free_position.may_be_registered_or_closed(approvals) {
            msg!(
                "{:?} is not authorized to register {:?}",
                approvals,
                free_position
            );
            return err!(ErrorCode::InvalidPositionOwner);
        }

        Ok(key)
    }

    /// Free the space from a previously registered position no longer needed
    pub fn unregister_position(
        &mut self,
        mint: &Pubkey,
        account: &Pubkey,
        approvals: &[Approver],
    ) -> AnchorResult<()> {
        let removed = self.position_list_mut().remove(mint, account)?;

        if !removed.may_be_registered_or_closed(approvals) {
            msg!("{:?} is not authorized to close {:?}", approvals, removed);
            return err!(ErrorCode::InvalidPositionOwner);
        }
        if removed.balance != 0 {
            return err!(ErrorCode::CloseNonZeroPosition);
        }
        if removed.flags.contains(AdapterPositionFlags::REQUIRED) {
            return err!(ErrorCode::CloseRequiredPosition);
        }

        Ok(())
    }

    pub fn refresh_position_metadata(
        &mut self,
        mint: &Pubkey,
        kind: TokenKind,
        value_modifier: u16,
        max_staleness: u64,
    ) -> Result<AccountPosition, ErrorCode> {
        let position = match self.position_list_mut().get_mut(mint) {
            None => return Err(ErrorCode::PositionNotRegistered),
            Some(p) => p,
        };

        position.kind = kind.into_integer();
        position.value_modifier = value_modifier;
        position.max_staleness = max_staleness;

        Ok(*position)
    }

    pub fn get_position_key(&self, mint: &Pubkey) -> Option<AccountPositionKey> {
        self.position_list().get_key(mint).copied()
    }

    pub fn get_position(&mut self, mint: &Pubkey) -> Option<&AccountPosition> {
        self.position_list().get(mint)
    }

    pub fn get_position_mut(&mut self, mint: &Pubkey) -> Option<&mut AccountPosition> {
        self.position_list_mut().get_mut(mint)
    }

    /// faster than searching by mint only if you have the correct key
    /// slightly slower if you have the wrong key
    pub fn get_position_by_key(&self, key: &AccountPositionKey) -> Option<&AccountPosition> {
        let list = self.position_list();
        let position = &list.positions[usize::try_from(key.index).unwrap()];

        if position.token == key.mint {
            Some(position)
        } else {
            list.get(&key.mint)
        }
    }

    /// faster than searching by mint only if you have the correct key
    /// slightly slower if you have the wrong key
    pub fn get_position_by_key_mut(
        &mut self,
        key: &AccountPositionKey,
    ) -> Option<&mut AccountPosition> {
        let list = self.position_list_mut();
        let position = &list.positions[usize::try_from(key.index).unwrap()];

        if position.token == key.mint {
            Some(&mut list.positions[usize::try_from(key.index).unwrap()])
        } else {
            list.get_mut(&key.mint)
        }
    }

    /// Change the balance for a position
    pub fn set_position_balance(
        &mut self,
        mint: &Pubkey,
        account: &Pubkey,
        balance: u64,
    ) -> Result<AccountPosition, ErrorCode> {
        let position = self.position_list_mut().get_mut(mint).require()?;

        if position.address != *account {
            return Err(ErrorCode::PositionNotRegistered);
        }

        position.set_balance(balance);

        Ok(*position)
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

        if info.required_collateral > info.effective_collateral || info.past_due {
            let due_status = match info.past_due {
                true => "overdue",
                false => "not overdue",
            };

            msg!(
                "account is unhealthy: K_e = {}, K_r = {} ({})",
                info.effective_collateral,
                info.required_collateral,
                due_status
            );
            return err!(ErrorCode::Unhealthy);
        }

        Ok(())
    }

    /// Check that the overall health of the account is *not* acceptable.
    pub fn verify_unhealthy_positions(&self) -> AnchorResult<()> {
        let info = self.valuation()?;

        if !info.stale_collateral_list.is_empty() {
            for (position_token, error) in info.stale_collateral_list {
                msg!("stale position {}: {}", position_token, error)
            }
            return Err(error!(ErrorCode::StalePositions));
        }

        match info.required_collateral > info.effective_collateral {
            true => Ok(()),
            false if info.past_due => Ok(()),
            false => err!(ErrorCode::Healthy),
        }
    }

    /// Check if the given address is the current authority for this margin account
    pub fn verify_authority(&self, authority: Pubkey) -> Result<(), ErrorCode> {
        if self.is_liquidating() {
            if authority == self.owner {
                return Err(ErrorCode::Liquidating);
            } else if authority != self.liquidator {
                return Err(ErrorCode::UnauthorizedLiquidator);
            }
        } else if authority != self.owner {
            return Err(ErrorCode::UnauthorizedInvocation);
        }

        Ok(())
    }

    pub fn valuation(&self) -> AnchorResult<Valuation> {
        let timestamp = sys().unix_timestamp();

        let mut past_due = false;
        let mut liabilities = Number128::ZERO;
        let mut required_collateral = Number128::ZERO;
        let mut weighted_collateral = Number128::ZERO;
        let mut stale_collateral_list = vec![];
        let mut equity = Number128::ZERO;

        for position in self.positions() {
            if position.balance == 0 {
                continue;
            }
            let kind = position.kind();
            let stale_reason = {
                let balance_age = timestamp - position.balance_timestamp;
                let price_quote_age = timestamp - position.price.timestamp;

                if !position.price.is_valid() {
                    // collateral with bad prices
                    Some(ErrorCode::InvalidPrice)
                } else if position.max_staleness > 0 && balance_age > position.max_staleness {
                    // outdated balance
                    Some(ErrorCode::OutdatedBalance)
                } else if price_quote_age > MAX_PRICE_QUOTE_AGE {
                    // outdated price
                    Some(ErrorCode::OutdatedPrice)
                } else {
                    None
                }
            };

            match (kind, stale_reason) {
                (TokenKind::Claim, None) => {
                    if position.balance > 0
                        && position.flags.contains(AdapterPositionFlags::PAST_DUE)
                    {
                        past_due = true;
                    }

                    equity -= position.value();
                    liabilities += position.value();
                    required_collateral += position.required_collateral_value();
                }
                (TokenKind::Claim, Some(error)) => {
                    msg!("claim position is stale: {:?}", position);
                    return Err(error!(error));
                }

                (TokenKind::AdapterCollateral | TokenKind::Collateral, None) => {
                    equity += position.value();
                    weighted_collateral += position.collateral_value();
                }
                (TokenKind::AdapterCollateral | TokenKind::Collateral, Some(e)) => {
                    stale_collateral_list.push((position.token, e));
                }
            }
        }

        Ok(Valuation {
            equity,
            liabilities,
            past_due,
            required_collateral,
            weighted_collateral,
            effective_collateral: weighted_collateral - liabilities,
            stale_collateral_list,
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

#[derive(PartialEq, Eq, Debug)]
pub enum Approver {
    /// Do not include this unless the transaction was signed by the margin account authority
    MarginAccountAuthority,

    /// Do not include this unless the request came from an adapter's return data
    Adapter(Pubkey),
}

/// State of an in-progress liquidation
#[account(zero_copy)]
#[repr(C, align(8))]
pub struct LiquidationState {
    /// The state object
    pub state: Liquidation,
}

#[repr(C)]
#[derive(Zeroable, Pod, AnchorDeserialize, AnchorSerialize, Debug, Default, Clone, Copy)]
pub struct Liquidation {
    /// time that liquidate_begin initialized this liquidation
    pub start_time: i64,

    /// cumulative change in equity caused by invocations during the liquidation so far
    /// negative if equity is lost
    pub equity_change: i128,

    /// lowest amount of equity change that is allowed during invoke steps
    /// typically negative or zero
    /// if equity_change goes lower than this number, liquidate_invoke should fail
    pub min_equity_change: i128,
}

impl Liquidation {
    pub fn new(start_time: i64, min_equity_change: Number128) -> Self {
        Self {
            start_time,
            equity_change: 0,
            min_equity_change: min_equity_change.to_i128(),
        }
    }

    pub fn start_time(&self) -> i64 {
        self.start_time
    }

    pub fn equity_change_mut(&mut self) -> &mut Number128 {
        bytemuck::cast_mut(&mut self.equity_change)
    }

    pub fn equity_change(&self) -> &Number128 {
        bytemuck::cast_ref(&self.equity_change)
    }

    pub fn min_equity_change(&self) -> Number128 {
        Number128::from_i128(self.min_equity_change)
    }
}

#[derive(Debug, Clone)]
pub struct Valuation {
    /// The net asset value for all positions registered in this account, ignoring collateral weights and max leverage
    pub equity: Number128,

    /// The total liability value for all claims, ignoring max leverage.
    pub liabilities: Number128,

    /// The amount of collateral that is required to cover price risk exposure from claim positions
    pub required_collateral: Number128,

    /// The total dollar value counted towards collateral from all deposits
    pub weighted_collateral: Number128,

    /// weighted_collateral minus debt. the remaining portion of collateral allocated for required_collateral after deposits and borrows offset
    pub effective_collateral: Number128,

    /// Errors that resulted in collateral positions from being excluded from collateral and equity totals
    stale_collateral_list: Vec<(Pubkey, ErrorCode)>,

    /// at least one position is past due and must be repaid immediately
    past_due: bool,
}

impl Valuation {
    pub fn available_collateral(&self) -> Number128 {
        self.effective_collateral - self.required_collateral
    }

    pub fn past_due(&self) -> bool {
        self.past_due
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        syscall::{sys, thread_local_mock::mock_stack_height, Sys},
        util::Invocation,
    };

    use super::*;
    use itertools::Itertools;
    use serde_test::{assert_ser_tokens, Token};

    fn create_position_input(margin_address: &Pubkey) -> (Pubkey, Pubkey) {
        let token = Pubkey::new_unique();
        let (address, _) =
            Pubkey::find_program_address(&[margin_address.as_ref(), token.as_ref()], &crate::id());
        (token, address)
    }

    #[test]
    fn margin_account_debug() {
        let mut invocation = Invocation::default();
        for i in [0, 1, 2, 4, 7] {
            mock_stack_height(Some(i));
            invocation.start();
        }
        let mut acc = MarginAccount {
            version: 1,
            bump_seed: [0],
            user_seed: [0; 2],
            reserved0: [0; 3],
            owner: Pubkey::default(),
            liquidation: Pubkey::default(),
            liquidator: Pubkey::default(),
            invocation,
            positions: [0; 7432],
        };
        let output = "MarginAccount {
            version: 1,
            bump_seed: [0],
            user_seed: [0, 0],
            reserved0: [0, 0, 0],
            invocation: Invocation {
                caller_heights: BitSet(0b10010111)
            },
            owner: 11111111111111111111111111111111,
            liquidation: 11111111111111111111111111111111,
            liquidator: 11111111111111111111111111111111,
            positions: []
        }"
        .split_whitespace()
        .join(" ");
        assert_eq!(&output, &format!("{acc:?}"));

        // use a non-default pubkey
        let key = crate::id();
        let approvals = &[Approver::MarginAccountAuthority];
        acc.register_position(
            key,
            2,
            key,
            key,
            TokenKind::Collateral,
            5000,
            1000,
            approvals,
        )
        .unwrap();
        let position = "AccountPosition {
            token: JPMRGNgRk3w2pzBM1RLNBnpGxQYsFQ3yXKpuk4tTXVZ,
            address: JPMRGNgRk3w2pzBM1RLNBnpGxQYsFQ3yXKpuk4tTXVZ,
            adapter: JPMRGNgRk3w2pzBM1RLNBnpGxQYsFQ3yXKpuk4tTXVZ,
            value: \"0.0\",
            balance: 0,
            balance_timestamp: 0,
            price: PriceInfo {
                value: 0,
                timestamp: 0,
                exponent: 0,
                is_valid: 0,
                _reserved: [0, 0, 0]
            },
            kind: Collateral,
            exponent: -2,
            value_modifier: 5000,
            max_staleness: 1000
        }"
        .split_whitespace()
        .join(" ");
        let output = output.replace("positions: []", &format!("positions: [{}]", position));
        assert_eq!(&output, &format!("{:?}", acc));
    }

    #[test]
    fn margin_account_serialize() {
        let account = MarginAccount {
            version: 1,
            bump_seed: [0],
            user_seed: [0; 2],
            reserved0: [0; 3],
            owner: Pubkey::default(),
            liquidation: Pubkey::default(),
            liquidator: Pubkey::default(),
            invocation: Invocation::default(),
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
        let position = AccountPosition::default();

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
                Token::Str("Collateral"),
                Token::Str("exponent"),
                Token::I16(0),
                Token::Str("valueModifier"),
                Token::U16(0),
                Token::Str("maxStaleness"),
                Token::U64(0),
                Token::StructEnd,
            ],
        )
    }

    #[test]
    fn valuation_fails_on_stale_claim_with_balance() {
        let mut margin_account = MarginAccount {
            version: 1,
            bump_seed: [0],
            user_seed: [0; 2],
            reserved0: [0; 3],
            owner: Pubkey::new_unique(),
            liquidation: Pubkey::default(),
            liquidator: Pubkey::default(),
            invocation: Invocation::default(),
            positions: [0; 7432],
        };
        let pos = register_position(&mut margin_account, 0, TokenKind::Claim);
        margin_account.set_position_balance(&pos, &pos, 1).unwrap();

        assert!(margin_account.valuation().is_err());
    }

    #[test]
    fn valuation_succeeds_ignoring_stale_adapter_collateral_with_balance() {
        let mut margin_account = MarginAccount {
            version: 1,
            bump_seed: [0],
            user_seed: [0; 2],
            reserved0: [0; 3],
            owner: Pubkey::new_unique(),
            liquidation: Pubkey::default(),
            liquidator: Pubkey::default(),
            invocation: Invocation::default(),
            positions: [0; 7432],
        };

        let pos = register_position(&mut margin_account, 0, TokenKind::AdapterCollateral);

        margin_account.set_position_balance(&pos, &pos, 1).unwrap();
        let valuation = margin_account.valuation().unwrap();
        assert_eq!(valuation.effective_collateral, Number128::ZERO);
        assert_eq!(valuation.equity, Number128::ZERO);

        margin_account
            .set_position_price(
                &pos,
                &PriceInfo {
                    value: 1,
                    timestamp: sys().unix_timestamp(),
                    exponent: 2,
                    is_valid: 1,
                    _reserved: Default::default(),
                },
            )
            .unwrap();
        let valuation = margin_account.valuation().unwrap();
        assert_eq!(valuation.effective_collateral, Number128::ONE * 100);
        assert_eq!(valuation.equity, Number128::ONE);
    }

    #[test]
    fn test_mutate_positions() {
        let margin_address = Pubkey::new_unique();
        let adapter = Pubkey::new_unique();
        let mut margin_account = MarginAccount {
            version: 1,
            bump_seed: [0],
            user_seed: [0; 2],
            reserved0: [0; 3],
            owner: Pubkey::new_unique(),
            liquidation: Pubkey::default(),
            liquidator: Pubkey::default(),
            invocation: Invocation::default(),
            positions: [0; 7432],
        };
        let user_approval = &[Approver::MarginAccountAuthority];
        let adapter_approval = &[Approver::MarginAccountAuthority, Approver::Adapter(adapter)];

        // // Register a few positions, randomise the order
        let (token_e, address_e) = create_position_input(&margin_address);
        let (token_a, address_a) = create_position_input(&margin_address);
        let (token_d, address_d) = create_position_input(&margin_address);
        let (token_c, address_c) = create_position_input(&margin_address);
        let (token_b, address_b) = create_position_input(&margin_address);

        margin_account
            .register_position(
                token_a,
                6,
                address_a,
                adapter,
                TokenKind::Collateral,
                0,
                0,
                user_approval,
            )
            .unwrap();

        margin_account
            .register_position(
                token_b,
                6,
                address_b,
                adapter,
                TokenKind::Claim,
                0,
                0,
                adapter_approval,
            )
            .unwrap();

        margin_account
            .register_position(
                token_c,
                6,
                address_c,
                adapter,
                TokenKind::Collateral,
                0,
                0,
                user_approval,
            )
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
            .unregister_position(&token_a, &address_a, user_approval)
            .unwrap();
        assert_eq!(margin_account.positions().count(), 2);
        margin_account
            .unregister_position(&token_b, &address_b, adapter_approval)
            .unwrap();
        assert_eq!(margin_account.positions().count(), 1);

        margin_account
            .register_position(
                token_e,
                9,
                address_e,
                adapter,
                TokenKind::Collateral,
                0,
                100,
                user_approval,
            )
            .unwrap();
        assert_eq!(margin_account.positions().count(), 2);

        margin_account
            .register_position(
                token_d,
                9,
                address_d,
                adapter,
                TokenKind::Collateral,
                0,
                100,
                user_approval,
            )
            .unwrap();
        assert_eq!(margin_account.positions().count(), 3);

        // It should not be possible to unregister mismatched token & position
        assert!(margin_account
            .unregister_position(&token_c, &address_b, user_approval)
            .is_err());

        margin_account
            .unregister_position(&token_c, &address_c, user_approval)
            .unwrap();
        margin_account
            .unregister_position(&token_e, &address_e, user_approval)
            .unwrap();
        margin_account
            .unregister_position(&token_d, &address_d, user_approval)
            .unwrap();

        // There should be no positions left
        assert_eq!(margin_account.positions().count(), 0);
        assert_eq!(margin_account.positions, [0; 7432]);
    }

    #[test]
    fn registering_adapter_collateral_requires_adapter_and_owner_approval() {
        let margin_address = Pubkey::new_unique();
        let adapter = Pubkey::new_unique();
        let mut margin_account = MarginAccount {
            version: 1,
            bump_seed: [0],
            user_seed: [0; 2],
            reserved0: [0; 3],
            owner: Pubkey::new_unique(),
            liquidation: Pubkey::default(),
            liquidator: Pubkey::default(),
            invocation: Invocation::default(),
            positions: [0; 7432],
        };
        let (token_a, address_a) = create_position_input(&margin_address);
        let (token_b, address_b) = create_position_input(&margin_address);
        let (token_c, address_c) = create_position_input(&margin_address);
        let (token_d, address_d) = create_position_input(&margin_address);

        margin_account
            .register_position(
                token_a,
                6,
                address_a,
                adapter,
                TokenKind::AdapterCollateral,
                0,
                0,
                &[],
            )
            .unwrap_err();
        margin_account
            .register_position(
                token_b,
                6,
                address_b,
                adapter,
                TokenKind::AdapterCollateral,
                0,
                0,
                &[Approver::MarginAccountAuthority],
            )
            .unwrap_err();
        margin_account
            .register_position(
                token_c,
                6,
                address_c,
                adapter,
                TokenKind::AdapterCollateral,
                0,
                0,
                &[Approver::Adapter(adapter)],
            )
            .unwrap_err();
        margin_account
            .register_position(
                token_d,
                6,
                address_d,
                adapter,
                TokenKind::AdapterCollateral,
                0,
                0,
                &[Approver::MarginAccountAuthority, Approver::Adapter(adapter)],
            )
            .unwrap();
    }

    #[test]
    fn adapter_collateral() {
        let margin_address = Pubkey::new_unique();
        let adapter = Pubkey::new_unique();
        let mut margin_account = MarginAccount {
            version: 1,
            bump_seed: [0],
            user_seed: [0; 2],
            reserved0: [0; 3],
            owner: Pubkey::new_unique(),
            liquidation: Pubkey::default(),
            liquidator: Pubkey::default(),
            invocation: Invocation::default(),
            positions: [0; 7432],
        };
        let (token, address) = create_position_input(&margin_address);

        margin_account
            .register_position(
                token,
                6,
                address,
                adapter,
                TokenKind::AdapterCollateral,
                0,
                0,
                &[Approver::MarginAccountAuthority, Approver::Adapter(adapter)],
            )
            .unwrap();
    }

    #[test]
    fn margin_account_past_due() {
        let mut acc = MarginAccount {
            version: 1,
            bump_seed: [0],
            user_seed: [0; 2],
            reserved0: [0; 3],
            owner: Pubkey::default(),
            liquidation: Pubkey::default(),
            liquidator: Pubkey::default(),
            invocation: Invocation::default(),
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
        try_register_position(acc, index, kind).unwrap()
    }

    fn try_register_position(
        acc: &mut MarginAccount,
        index: u8,
        kind: TokenKind,
    ) -> AnchorResult<Pubkey> {
        let key = Pubkey::find_program_address(&[&[index]], &crate::id()).0;
        let mut approvals = vec![Approver::MarginAccountAuthority];

        match kind {
            TokenKind::Claim | TokenKind::AdapterCollateral => {
                approvals.push(Approver::Adapter(key))
            }
            _ => (),
        }

        acc.register_position(key, 2, key, key, kind, 10000, 0, &approvals)?;

        Ok(key)
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
                timestamp: sys().unix_timestamp(),
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

    #[test]
    fn margin_account_no_more_than_24_positions() {
        let mut account = blank_account();
        for i in 0..24 {
            try_register_position(&mut account, i, TokenKind::Collateral).unwrap();
        }
        try_register_position(&mut account, 24, TokenKind::Collateral).unwrap_err();
    }

    #[test]
    fn margin_account_32_positions_with_liquidator() {
        let mut account = blank_account();
        account.liquidation = pda(234);
        for i in 0..30 {
            try_register_position(&mut account, i, TokenKind::Collateral).unwrap();
        }
    }

    #[test]
    fn margin_account_authority() {
        let mut account = blank_account();
        account.owner = pda(0);
        account.liquidator = pda(1);
        account.verify_authority(pda(0)).unwrap();
        account.verify_authority(pda(1)).unwrap_err();
        account.verify_authority(pda(2)).unwrap_err();
        account.verify_authority(Pubkey::default()).unwrap_err();
    }

    #[test]
    fn margin_account_authority_during_liquidation() {
        let mut account = blank_account();
        account.owner = pda(0);
        account.liquidator = pda(1);
        account.liquidation = pda(2);
        account.verify_authority(pda(0)).unwrap_err();
        account.verify_authority(pda(1)).unwrap();
        account.verify_authority(pda(2)).unwrap_err();
        account.verify_authority(Pubkey::default()).unwrap_err();
    }

    fn pda(index: u8) -> Pubkey {
        Pubkey::find_program_address(&[&[index]], &crate::id()).0
    }

    fn blank_account() -> MarginAccount {
        MarginAccount {
            version: 1,
            bump_seed: [0],
            user_seed: [0; 2],
            reserved0: [0; 3],
            owner: Pubkey::default(),
            liquidation: Pubkey::default(),
            liquidator: Pubkey::default(),
            invocation: Invocation::default(),
            positions: [0; 7432],
        }
    }
}
