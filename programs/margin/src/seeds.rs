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

#[constant]
pub const TOKEN_CONFIG_SEED: &[u8] = b"token-config";

#[constant]
pub const ADAPTER_CONFIG_SEED: &[u8] = b"adapter-config";

#[constant]
#[deprecated(note = "liquidator config is now in account at the pda derived from PERMIT_SEED")]
pub const LIQUIDATOR_CONFIG_SEED: &[u8] = b"liquidator-config";

#[constant]
pub const PERMIT_SEED: &[u8] = b"permit";
