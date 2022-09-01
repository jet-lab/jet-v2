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

use std::ops::{Deref, DerefMut};

use anchor_lang::prelude::*;

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

/// The isolation boundary for protocol resources
#[account]
pub struct Airspace {
    /// The address allowed to make administrative changes to this airspace.
    pub authority: Pubkey,

    /// If true, resources within the airspace should be restricted to only users that receive
    /// permission from an authorized regulator. If false, any user may request a permit without
    /// the need for any authorization.
    pub is_restricted: bool,

    /// Configuration for specific resources in the airspace to interpret
    pub directives: Directives,
}

declare_account_size!(Airspace, 304);

/// Permission for an address to issue permits to other addresses to interact with resources
/// in an airspace.
#[account]
pub struct AirspacePermitIssuerId {
    /// The relevant airspace for this regulator
    pub airspace: Pubkey,

    /// The address authorized to sign permits allowing users to create accounts
    /// within the airspace
    pub issuer: Pubkey,
}

declare_account_size!(AirspacePermitIssuerId, 128);

/// A permission given to a user address that enables them to use resources within an airspace.
#[account]
pub struct AirspacePermit {
    /// The address of the `Airspace` this permit applies to
    pub airspace: Pubkey,

    /// The owner of this permit, which is the address allowed to sign for any interactions
    /// with resources within the airspace (e.g. margin accounts, lending pools, etc)
    pub owner: Pubkey,

    /// The issuer of this permit
    pub issuer: Pubkey,
}

declare_account_size!(AirspacePermit, 128);

/// A global account specifying the current governing address for the protocol
#[account]
pub struct GovernorId {
    /// The governing address, which as authority to make configuration changes
    /// to the protocol, including all airspaces.
    pub governor: Pubkey,
}

declare_account_size!(GovernorId, 40);

/// The set of directives are configuration for resources in an airspace, which are managed
/// by the protocol governance. The authority for an airspace has no control over this
/// configuration.
#[derive(Default, AnchorDeserialize, AnchorSerialize, Debug, Clone, Copy)]
pub struct Directives {
    /// The fee applied to interest earned on margin pools.
    pub margin_pool_management_fee_rate: u16,
}

/// Account containing the default set of directives that airspaces will be created with
#[account]
pub struct DefaultDirectives {
    pub value: Directives,
}

declare_account_size!(DefaultDirectives, 256);

impl Deref for DefaultDirectives {
    type Target = Directives;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for DefaultDirectives {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
