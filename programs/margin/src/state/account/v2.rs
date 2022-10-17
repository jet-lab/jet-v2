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

//! Margin Account State V2
//!
//! The account state is comprised of two sections:
//!
//! +===========+==============+
//! |  Offset   |   Content    |
//! +===========+==============+
//! | 0 .. 256  | Header       |
//! +-----------+--------------+
//! |   256 ... | Position Map |
//! +-----------+--------------+
//!
//!  * Header
//!    
//!    The header is a fixed size of 256 bytes, with some of that space being unused and reserved
//!    for potential future usage. This contains any information/metadata about the account that's
//!    not specific to a position.
//!
//!  * Position Map
//!    
//!    The position map contains all of the information on positions owned by the account. The map
//!    itself is dynamically sized, fitting within whatever the capacity for the account memory is.
//!    This allows for position capacity in this account to be extended by increasing the underlying
//!    memory capacity.
//! 
//!    The map is keyed on the address of the token/mint for the asset being owned by the account. 
//!    The map state is stored as a red-black tree, the capacity is based on the available account
//!    memory divided by the size of each position node.

use anchor_lang::prelude::Pubkey;
use bytemuck::{Pod, Zeroable, Contiguous};

use jet_core::{tree::Tree, Reserved, Seeds};
use jet_proto_math::Number128;
use jet_proto_proc_macros::assert_size;

use crate::{AccountPosition, ErrorCode, Invocation, PriceInfo, AdapterPositionFlags};

use super::operations::MarginAccountOperations;

pub const VERSION: u8 = 2;
pub const SEED_PREFIX: &[u8] = b"margin-account";

#[assert_size(256)]
#[derive(Pod, Zeroable, Debug, Clone, Copy)]
#[repr(C)]
struct AccountHeader {
    pub version: u8,
    pub bump_seed: [u8; 1],
    pub user_seed: [u8; 2],

    /// Data an adapter can use to check what the margin program thinks about the current invocation
    /// Must normally be zeroed, except during an invocation.
    pub invocation: Invocation,

    pub _reserved0: Reserved<3>,

    /// The owner of this account, which generally has to sign for any changes to it
    pub owner: Pubkey,

    /// The airspace this account belongs in
    pub airspace: Pubkey,

    /// The state of an active liquidation for this account
    pub liquidation: Pubkey,

    /// The active liquidator for this account
    pub liquidator: Pubkey,

    pub _reserved1: Reserved<120>,
}

pub struct MarginAccountV2<'a> {
    header: &'a mut AccountHeader,
    positions: Tree<'a, Pubkey, Position>,
}

impl<'a> MarginAccountV2<'a> {
    /// Initialize the state for the margin account in the given buffer
    pub fn new(
        buffer: &'a mut [u8],
        bump_seed: u8,
        user_seed: u16,
        owner: Pubkey,
        airspace: Pubkey,
    ) -> Result<MarginAccountV2<'a>, ErrorCode> {
        if buffer[0] != 0 {
            return Err(ErrorCode::AlreadyCreatedAccount);
        }

        let (header_buf, position_buf) = buffer.split_at_mut(std::mem::size_of::<AccountHeader>());
        let header: &mut AccountHeader = bytemuck::from_bytes_mut(header_buf);
        let positions = Tree::new(position_buf).unwrap();

        header.version = VERSION;
        header.bump_seed[0] = bump_seed;
        header.user_seed = user_seed.to_le_bytes();
        header.owner = owner;
        header.airspace = airspace;

        Ok(Self { header, positions })
    }

    /// Load a previously initialized account from a buffer
    pub fn load(buffer: &'a mut [u8]) -> Result<Self, ErrorCode> {
        if buffer[0] != VERSION {
            return Err(ErrorCode::InvalidAccount);
        }

        let (header_buf, position_buf) = buffer.split_at_mut(std::mem::size_of::<AccountHeader>());

        Ok(Self {
            header: bytemuck::from_bytes_mut(header_buf),
            positions: Tree::load(position_buf).unwrap(),
        })
    }
}

impl<'a> MarginAccountOperations for MarginAccountV2<'a> {
    fn signer_seeds(&self) -> Seeds {
        Seeds::new(&[
            SEED_PREFIX,
            self.header.owner.as_ref(),
            self.header.airspace.as_ref(),
            self.header.user_seed.as_ref(),
            self.header.bump_seed.as_ref(),
        ])
    }

    fn register_position(
            &mut self,
            config: &crate::TokenConfig,
            address: Pubkey,
        ) -> Result<AccountPosition, ErrorCode> {
        
    }
}

#[derive(Contiguous, Clone, Copy)]
#[repr(u8)]
enum PositionAdmin {
    Adapter,
    MarginPyth
}

#[assert_size(168)]
#[derive(Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct Position {
    admin: u8,

    /// Flags for this position
    pub flags: AdapterPositionFlags,

    _reserved: Reserved<30>,

    /// The address of the account holding the tokens.
    pub address: Pubkey,

    /// The address of the administrator for the position asset
    pub admin_address: Pubkey,

    /// The current value of this position
    pub value: Number128,

    /// The amount of tokens held in this position
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
    pub value_modifier: u16,

    /// The max staleness for the account balance (seconds)
    pub max_staleness: u64,
}