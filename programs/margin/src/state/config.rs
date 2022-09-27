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

use anchor_lang::prelude::*;
use bytemuck::Contiguous;

use crate::ErrorCode;

/// Description of the token's usage
#[derive(AnchorSerialize, AnchorDeserialize, Contiguous, Eq, PartialEq, Clone, Copy, Debug)]
#[repr(u32)]
pub enum TokenKind {
    /// The token can be used as collateral
    Collateral = 1,

    /// The token represents a debt that needs to be repaid
    Claim,

    /// The token balance is managed by a trusted adapter to represent the amount of collateral
    /// custodied by that adapter. The token account is owned by the adapter. Collateral
    /// is accessed through instructions to the adapter.
    AdapterCollateral,
}

impl Default for TokenKind {
    fn default() -> TokenKind {
        Self::Collateral
    }
}

impl From<jet_metadata::TokenKind> for TokenKind {
    fn from(kind: jet_metadata::TokenKind) -> Self {
        match kind {
            jet_metadata::TokenKind::NonCollateral => Self::Collateral,
            jet_metadata::TokenKind::Collateral => Self::Collateral,
            jet_metadata::TokenKind::Claim => Self::Claim,
            jet_metadata::TokenKind::AdapterCollateral => Self::AdapterCollateral,
        }
    }
}

/// The configuration account specifying parameters for a token when used
/// in a position within a margin account.
#[account]
#[derive(Debug, Eq, PartialEq)]
pub struct TokenConfig {
    /// The mint for the token
    pub mint: Pubkey,

    /// The mint for the underlying token represented, if any
    pub underlying_mint: Pubkey,

    /// The space this config is valid within
    pub airspace: Pubkey,

    /// The administrator of this token, which has the authority to provide information
    /// about (e.g. prices) and otherwise modify position states for these tokens.
    pub admin: TokenAdmin,

    /// Description of this token
    ///
    /// This determines the way the margin program values a token as a position in a
    /// margin account.
    pub token_kind: TokenKind,

    /// A modifier to adjust the token value, based on the kind of token
    pub value_modifier: u16,

    /// The maximum staleness (seconds) that's acceptable for balances of this token
    pub max_staleness: u64,
}

impl TokenConfig {
    pub const SPACE: usize = 8 + 2 + std::mem::size_of::<Self>();

    pub fn validate(&self) -> Result<()> {
        if self.underlying_mint == Pubkey::default() {
            msg!("the underlying mint must be set");
            return err!(ErrorCode::InvalidConfig);
        }

        Ok(())
    }

    pub fn adapter_program(&self) -> Option<Pubkey> {
        match self.admin {
            TokenAdmin::Adapter(address) => Some(address),
            _ => None,
        }
    }

    pub fn oracle(&self) -> Option<TokenOracle> {
        match self.admin {
            TokenAdmin::Margin { oracle } => Some(oracle),
            _ => None,
        }
    }
}

/// Information about where to find the oracle data for a token
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub enum TokenOracle {
    Pyth {
        /// The pyth address containing price information for a token.
        price: Pubkey,

        /// The pyth address with product information for a token
        product: Pubkey,
    },
}

/// Description of which program administers a token
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub enum TokenAdmin {
    /// This margin program administers the token directly
    Margin {
        /// An oracle that can be used to collect price information for a token
        oracle: TokenOracle,
    },

    /// The token is administered by the given adapter program
    ///
    /// The adapter is responsible for providing price information for the token.
    Adapter(Pubkey),
}

/// Configuration for allowed liquidators
#[account]
#[derive(Default, Debug, Eq, PartialEq)]
pub struct LiquidatorConfig {
    /// The airspace this liquidator is being configured to act within
    pub airspace: Pubkey,

    /// The address of the liquidator allowed to act
    pub liquidator: Pubkey,
}

impl LiquidatorConfig {
    pub const SPACE: usize = 8 + std::mem::size_of::<Self>();
}

/// Configuration for allowed adapters
#[account]
#[derive(Default, Debug, Eq, PartialEq)]
pub struct AdapterConfig {
    /// The airspace this adapter can be used in
    pub airspace: Pubkey,

    /// The program address allowed to be called as an adapter
    pub adapter_program: Pubkey,
}

impl AdapterConfig {
    pub const SPACE: usize = 8 + std::mem::size_of::<Self>();
}
