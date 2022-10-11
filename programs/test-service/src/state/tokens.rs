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

/// Information about a token created by this testing service
#[account]
pub struct TokenInfo {
    pub bump_seed: u8,
    pub symbol: String,
    pub name: String,
    pub authority: Pubkey,
    pub oracle_authority: Pubkey,
    pub mint: Pubkey,
    pub pyth_price: Pubkey,
    pub pyth_product: Pubkey,
    pub max_request_amount: u64,
}

impl TokenInfo {
    pub const SIZE: usize = 1024;
}
