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

use bytemuck::{Pod, Zeroable};

use jet_proto_proc_macros::assert_size;

use crate::{AccountPosition, AccountPositionKey, Invocation, SignerSeeds};

#[constant]
pub const MARGIN_ACCOUNT_SEED: &[u8; SEED_LEN] = b"margin-account";

const SEED_LEN: usize = 14;

#[account(zero_copy)]
#[repr(C)]
// bytemuck requires a higher alignment than 1 for unit tests to run.
#[cfg_attr(not(target_arch = "bpf"), repr(align(8)))]
pub struct MarginAccountV1 {
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

#[assert_size(80)]
#[derive(Pod, Zeroable, Default, Clone, Copy)]
#[repr(C)]
pub struct MarginAccountHeader {
    /// The version of the account structure
    pub version: u8,

    /// The bump seed used to derive the address for the account
    pub bump_seed: [u8; 1],

    /// The owner-provided seed used to derive the address for the account
    pub user_seed: [u8; 2],

    /// The capacity or max size of the position list
    pub position_list_capacity: u16,

    /// Data an adapter can use to check what the margin program thinks about the current invocation
    /// Must normally be zeroed, except during an invocation.
    pub invocation: Invocation,

    /// Unused
    pub _reserved0: u8,

    /// The owner of this account, which generally has to sign for any changes to it
    pub owner: Pubkey,

    /// The airspace this account is a part of
    pub airspace: Pubkey,

    /// Unused
    pub _reserved1: [u8; 8]
}

#[assert_size(64)]
#[derive(Pod, Zeroable, AnchorSerialize, AnchorDeserialize, Default, Clone, Copy)]
#[repr(C)]
pub struct MarginAccountMetadata {
    /// The state of an active liquidation for this account
    pub liquidation: Pubkey,

    /// The active liquidator for this account
    pub liquidator: Pubkey,
}

impl MarginAccountHeader {
    fn position_list_offset(&self) -> usize {
        0
    }

    fn position_map_offset(&self) -> usize {
        self.position_list_capacity as usize * std::mem::size_of::<AccountPosition>()
    }

    fn metadata_offset(&self) -> usize {
        let capacity = self.position_list_capacity as usize;
        let position_list_len = capacity * std::mem::size_of::<AccountPosition>();
        let position_map_len = capacity * std::mem::size_of::<AccountPositionKey>();

        position_list_len + position_map_len
    }
}

pub struct MarginAccount<'info> {
    data: &'info mut [u8],
    header: &'info mut MarginAccountHeader,
}

impl<'info> MarginAccount<'info> {
    pub fn version(&self) -> u8 {
        self.header.version
    }

    pub fn owner(&self) -> &Pubkey {
        &self.header.owner
    }

    pub fn airspace(&self) -> &Pubkey {
        &self.header.airspace
    }

    pub fn metadata(&self) -> &MarginAccountMetadata {
        let offset = self.header.metadata_offset();

        bytemuck::from_bytes(
            &self.data[offset..offset + std::mem::size_of::<MarginAccountMetadata>()],
        )
    }

    pub fn metadata_mut(&mut self) -> &mut MarginAccountMetadata {
        let offset = self.header.metadata_offset();

        bytemuck::from_bytes_mut(
            &mut self.data[offset..offset + std::mem::size_of::<MarginAccountMetadata>()],
        )
    }
}