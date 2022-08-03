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

use std::{collections::HashSet, sync::Arc};

use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token_swap::state::SwapV1;

#[derive(Debug, Clone, Copy)]
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

impl SwapPool {
    /// Get all swap pools that contain pairs of supported mints
    pub async fn get_pools(
        rpc: &Arc<dyn SolanaRpcClient>,
        supported_mints: HashSet<Pubkey>,
        swap_program: Pubkey,
    ) -> anyhow::Result<Vec<Self>> {
        let size = SwapV1::LEN + 1;
        let accounts = rpc
            .get_program_accounts(&swap_program, Some(size))
            .await
            .unwrap();
        let swap_pools = accounts
            .iter()
            .filter_map(|(address, account)| {
                let swap = SwapV1::unpack(&account.data[1..]).ok()?;
                // Check if both tokens of the swap pool are supported
                if supported_mints.contains(&swap.token_a_mint)
                    && supported_mints.contains(&swap.token_b_mint)
                {
                    Some(SwapPool {
                        pool: *address,
                        pool_authority: Pubkey::find_program_address(
                            &[address.as_ref(), &[swap.nonce]],
                            &swap_program,
                        )
                        .0,
                        mint_a: swap.token_a_mint,
                        mint_b: swap.token_b_mint,
                        token_a: swap.token_a,
                        token_b: swap.token_b,
                        fee_account: swap.pool_fee_account,
                        pool_mint: swap.pool_mint,
                        program: swap_program,
                    })
                } else {
                    None
                }
            })
            .collect();
        Ok(swap_pools)
    }
}
