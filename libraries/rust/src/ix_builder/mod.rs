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

use solana_sdk::pubkey::Pubkey;

mod control;
mod margin;
mod margin_pool;
mod margin_swap;

pub use control::*;
pub use margin::*;
pub use margin_pool::*;
pub use margin_swap::*;

/// Get the address of a [jet_metadata] account.
///
/// Metadata addresses are PDAs of various metadata types. Refer to `jet_metadata` for
/// the different account types.
pub fn get_metadata_address(address: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[address.as_ref()], &jet_metadata::ID).0
}
