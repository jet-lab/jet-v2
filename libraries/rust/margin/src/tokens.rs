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

/// The addresses of a token's product and price details.
/// These are based on Pyth oracle accounts.
#[derive(Clone, Copy)]
pub struct TokenOracle {
    /// Address of the Pyth price account
    pub price: Pubkey,
    /// Address of the Pyth product account
    pub product: Pubkey,
}

/// A summarised struct of price information, used as a convenience when the
/// full Pyth price information is not required.
#[derive(Clone, Copy)]
pub struct TokenPrice {
    /// Token price
    pub price: i64,
    /// Exponent of the price
    pub exponent: i32,
    /// Confidence interval of the price, in the same units as the price.
    /// If a price is 100 and confidence is 2, the price range is 98 - 102.
    pub confidence: u64,
    /// Token time-weighted average price
    pub twap: u64,
}
