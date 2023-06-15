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

mod airspace;
/// invoke adapters through margin with minimal dependencies.
/// lighter-weight and more versatile alternative to MarginTxBuilder.
mod invoke_context;
/// invoke margin-pool through a margin account.
mod invoke_pool;
/// invoke margin-swap through a margin account.
mod invoke_swap;
mod user;

pub use airspace::*;
pub use invoke_context::*;
pub use invoke_pool::*;
pub use invoke_swap::*;
pub use user::*;
