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

//! The spl swap module gets all spl swap pools that contain pairs of supported mints

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anchor_lang::ToAccountMetas;
use jet_margin_swap::{accounts as ix_accounts, SwapRouteIdentifier};
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::{instruction::AccountMeta, program_pack::Pack, pubkey::Pubkey};
use spl_token_swap::state::SwapV1;

use crate::ix_builder::SwapAccounts;

use super::find_mint;

/// Addresses of an [`spl_token_swap`] compatible swap pool, required when using
/// [`jet_margin_swap`].
///
/// Supported pools are:
/// * spl_token_swap
/// * orca_v1
/// * orca_v2
#[derive(Debug, Clone, Copy)]
pub struct SplSwapPool {
    /// The address of the swap pool
    pub pool: Pubkey,
    /// The PDA of the pool authority, derived using the pool address and a nonce
    pub pool_authority: Pubkey,
    /// The mint of pool tokens that are minted when users supply pool liquidity
    pub pool_mint: Pubkey,
    /// The SPL token mint of one side of the pool
    pub mint_a: Pubkey,
    /// The SPL token mint of one side of the pool
    pub mint_b: Pubkey,
    /// The SPL token account that tokens are deposited into
    pub token_a: Pubkey,
    /// The SPL token account that tokens are deposited into
    pub token_b: Pubkey,
    /// The account that collects fees from the pool
    pub fee_account: Pubkey,
    /// The program of the pool, to distinguish between various supported pools
    pub program: Pubkey,
}

impl SplSwapPool {
    /// Get all swap pools that contain pairs of supported mints
    pub async fn get_pools(
        rpc: &Arc<dyn SolanaRpcClient>,
        supported_mints: &HashSet<Pubkey>,
        swap_program: Pubkey,
    ) -> anyhow::Result<HashMap<(Pubkey, Pubkey), Self>> {
        let size = SwapV1::LEN + 1;
        let accounts = rpc
            .get_program_accounts(&swap_program, Some(size))
            .await
            .unwrap();

        let mut pool_sizes = HashMap::with_capacity(accounts.len());
        for (swap_address, pool_account) in accounts {
            let swap = SwapV1::unpack(&pool_account.data[1..]);
            let swap = match swap {
                Ok(swap) => swap,
                Err(_) => continue,
            };

            if !swap.is_initialized {
                continue;
            }

            if supported_mints
                .get(&swap.token_a_mint)
                .and_then(|_| supported_mints.get(&swap.token_b_mint))
                .is_none()
            {
                continue;
            }

            // Get the pool tokens minted as a proxy of size
            let Ok(pool_mint) = find_mint(rpc, &swap.pool_mint).await else {
                continue;
            };
            let total_supply = pool_mint.supply;

            // Check if there is a pool, insert if none, replace if smaller
            pool_sizes
                .entry((swap.token_a_mint, swap.token_b_mint))
                .and_modify(|(e_pool, e_value)| {
                    if &total_supply > e_value {
                        // Replace with current pool
                        *e_pool = Self::from_swap_v1(swap_address, swap_program, &swap);
                        *e_value = total_supply;
                    }
                })
                .or_insert((
                    Self::from_swap_v1(swap_address, swap_program, &swap),
                    total_supply,
                ));
        }
        let swap_pools = pool_sizes
            .into_iter()
            .map(|(k, (p, _))| (k, p))
            .collect::<HashMap<_, _>>();

        Ok(swap_pools)
    }

    #[inline]
    /// Little helper to get a [SplSwapPool] from its on-chain rep
    fn from_swap_v1(swap_address: Pubkey, swap_program: Pubkey, swap: &SwapV1) -> Self {
        SplSwapPool {
            pool: swap_address,
            pool_authority: Pubkey::find_program_address(
                &[swap_address.as_ref(), &[swap.nonce]],
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
        }
    }
}

impl SwapAccounts for SplSwapPool {
    fn to_account_meta(&self, _authority: Pubkey) -> Vec<AccountMeta> {
        let (swap_authority, _) =
            Pubkey::find_program_address(&[self.pool.as_ref()], &self.program);

        ix_accounts::SplSwapInfo {
            swap_pool: self.pool,
            authority: swap_authority,
            vault_a: self.token_a,
            vault_b: self.token_b,
            token_mint: self.pool_mint,
            fee_account: self.fee_account,
            swap_program: self.program,
        }
        .to_account_metas(None)
    }

    fn pool_tokens(&self) -> (Pubkey, Pubkey) {
        (self.mint_a, self.mint_b)
    }

    fn route_type(&self) -> SwapRouteIdentifier {
        SwapRouteIdentifier::Spl
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }
}
