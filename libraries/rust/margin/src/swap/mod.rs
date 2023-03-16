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

//! The swap module interacts with supported swap protocols

use std::sync::Arc;

use anchor_lang::AccountDeserialize;
use anyhow::Result;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::pubkey::Pubkey;

pub mod openbook_swap;
pub mod saber_swap;
pub mod spl_swap;

// helper function to find mint account
pub(super) async fn find_mint(
    rpc: &Arc<dyn SolanaRpcClient>,
    address: &Pubkey,
) -> Result<anchor_spl::token::Mint> {
    let account = rpc.get_account(address).await?.unwrap();
    let data = &mut &account.data[..];
    let account = anchor_spl::token::Mint::try_deserialize_unchecked(data)?;

    Ok(account)
}
