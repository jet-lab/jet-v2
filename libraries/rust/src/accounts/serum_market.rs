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

/// Derived market accounts for [SerumMarketInfo]
#[derive(Clone)]
pub struct SerumMarketInfoAccounts {
    /// The address of the market info
    pub market_info: Pubkey,

    /// The address of the mint for the base order
    pub base_note_mint: Pubkey,

    /// The address of the mint for the quote order
    pub quote_note_mint: Pubkey,
}

impl SerumMarketInfoAccounts {
    /// Derive accounts.
    ///
    /// NOTE: The `program_id` parameter is a temporary solution until Serum trading is implemented
    pub fn derive(serum_market: Pubkey, program_id: &Pubkey) -> Self {
        let (market_info, _) = Pubkey::find_program_address(&[serum_market.as_ref()], program_id);

        let (base_note_mint, _) = Pubkey::find_program_address(
            &[market_info.as_ref(), b"base-mint".as_ref()],
            program_id,
        );

        let (quote_note_mint, _) = Pubkey::find_program_address(
            &[market_info.as_ref(), b"quote-mint".as_ref()],
            program_id,
        );

        Self {
            market_info,
            base_note_mint,
            quote_note_mint,
        }
    }
}
