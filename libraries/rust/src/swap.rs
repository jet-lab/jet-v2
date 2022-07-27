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

//! The margin swap module allows creating simulated swap pools
//! to aid in testing margin swaps.

use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Copy)]
pub struct SwapPool {
    pub pool: Pubkey,
    pub pool_authority: Pubkey,
    pub pool_mint: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub fee_account: Pubkey,
    pub program: Pubkey,
}
