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

use anchor_lang::prelude::Pubkey;

use jet_core::Seeds;
use jet_proto_math::Number128;

use crate::{ErrorCode, TokenConfig, PriceInfo};

use super::{positions::AccountPosition, v1};

/// Implements operations for margin accounts, and handles any version
/// differences with regards to the underlying account state.
pub struct MarginAccountOperator<'a> {
    state: MarginAccountVersion<'a>,
}

enum MarginAccountVersion<'a> {
    V1(&'a mut v1::MarginAccount),
}

/// Defines the operations that can be performed on margin account state
pub trait MarginAccountOperations {
    /// Get the seeds needed to sign for the margin account
    fn signer_seeds(&self) -> Seeds;

    /// Register a position and return the entry recorded for it
    fn register_position(
        &mut self,
        config: &TokenConfig,
        address: Pubkey,
    ) -> Result<&mut dyn MarginPosition, ErrorCode>;

    /// Free the space from a previously registered position no longer needed
    fn unregister_position(&mut self, mint: &Pubkey, address: &Pubkey) -> Result<(), ErrorCode>;
    
    fn get_position_mut(&mut self, mint: &Pubkey) -> Result<&mut dyn MarginPosition, ErrorCode>;
}

pub trait MarginAccountInfo {
    fn version(&self) -> u8;
    fn owner(&self) -> &Pubkey;
    fn airspace(&self) -> &Pubkey;
}

pub trait MarginPosition {
    /// The token/mint for the asset held by this position
    fn token(&self) -> &Pubkey;

    /// The address of the account containing the value held by this position
    fn address(&self) -> &Pubkey;

    /// The adapter with authority over this position
    fn adapter(&self) -> Option<&Pubkey>;

    /// The last recorded value of the position
    fn value(&self) -> &Number128;

    /// The last recorded price for the asset
    fn price(&self) -> &PriceInfo;

    /// The last recorded token balance held by this position
    fn balance(&self) -> u64;

    /// Update the current balance
    fn set_balance(&mut self, balance: u64);

    /// Update the price of the asset
    fn set_price(&mut self, price: &PriceInfo);
}