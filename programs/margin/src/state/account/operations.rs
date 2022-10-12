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

use crate::{util::Seeds, ErrorCode, TokenConfig};

use super::{positions::AccountPosition, structure::MarginAccountV1};

/// Implements operations for margin accounts, and handles any version
/// differences with regards to the underlying account state.
pub struct MarginAccountOperator<'a> {
    state: MarginAccountVersion<'a>,
}

enum MarginAccountVersion<'a> {
    V1(&'a mut MarginAccountV1),
}

/// Defines the common operations that can be performed on a margin account
trait MarginAccountOperations {
    /// Get the seeds needed to sign for the margin account
    fn signer_seeds(&self) -> Seeds;

    /// Register a position and return the entry recorded for it
    fn register_position(
        &mut self,
        config: &TokenConfig,
        address: Pubkey,
    ) -> Result<AccountPosition, ErrorCode>;

    /// Free the space from a previously registered position no longer needed
    fn unregister_position(&mut self, mint: &Pubkey, address: &Pubkey) -> Result<(), ErrorCode>;

    /// Update the configuration for an existing position
    fn refresh_position_config(
        &mut self,
        config: &TokenConfig,
    ) -> Result<AccountPosition, ErrorCode>;

    /// Iterates over all positions, applying a reducing function to return a
    /// new value.
    fn reduce_positions<F, R>(&self, action: F) -> R
    where
        F: FnMut(R, &AccountPosition) -> R,
        R: Default;
}
