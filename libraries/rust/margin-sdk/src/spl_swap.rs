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

use anchor_lang::AccountDeserialize;
use anyhow::Result;
use jet_proto_math::Number128;
use jet_rpc::solana_rpc_api::SolanaRpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token_swap::state::SwapV1;

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
        rpc: Arc<dyn SolanaRpcClient>,
        supported_mints: &HashSet<Pubkey>,
        swap_program: Pubkey,
    ) -> anyhow::Result<HashMap<(Pubkey, Pubkey), Self>> {
        let size = SwapV1::LEN + 1;
        let accounts = rpc
            .clone()
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

            let (mint_a, mint_b) = match (
                supported_mints.get(&swap.token_a_mint),
                supported_mints.get(&swap.token_b_mint),
            ) {
                (Some(a), Some(b)) => (a, b),
                _ => continue,
            };

            // Get the token balances of both sides
            let token_a = match find_token(rpc.clone(), &swap.token_a).await {
                Ok(val) => val,
                Err(_) => {
                    continue;
                }
            };
            let token_b = match find_token(rpc.clone(), &swap.token_b).await {
                Ok(val) => val,
                Err(_) => {
                    continue;
                }
            };

            let mint_a_info = find_mint(rpc.clone(), mint_a).await;
            let mint_b_info = find_mint(rpc.clone(), mint_b).await;

            let token_a_balance =
                Number128::from_decimal(token_a.amount, -(mint_a_info?.decimals as i32));
            let token_b_balance =
                Number128::from_decimal(token_b.amount, -(mint_b_info?.decimals as i32));

            let total_token = token_a_balance * token_b_balance;

            if !swap.is_initialized {
                continue;
            }

            // Check if there is a pool, insert if none, replace if smaller
            pool_sizes
                .entry((swap.token_a_mint, swap.token_b_mint))
                .and_modify(|(e_pool, e_value)| {
                    if &total_token > e_value {
                        // Replace with current pool
                        *e_pool = Self::from_swap_v1(swap_address, swap_program, &swap);
                        *e_value = total_token;
                    }
                })
                .or_insert((
                    Self::from_swap_v1(swap_address, swap_program, &swap),
                    total_token,
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

// helper function to find token account
async fn find_token(
    rpc: Arc<dyn SolanaRpcClient>,
    address: &Pubkey,
) -> Result<anchor_spl::token::TokenAccount> {
    let account = rpc.get_account(address).await?.unwrap();
    let data = &mut &account.data[..];
    let account = anchor_spl::token::TokenAccount::try_deserialize_unchecked(data)?;

    Ok(account)
}

// helper function to find mint account
async fn find_mint(
    rpc: Arc<dyn SolanaRpcClient>,
    address: &Pubkey,
) -> Result<anchor_spl::token::Mint> {
    let account = rpc.get_account(address).await?.unwrap();
    let data = &mut &account.data[..];
    let account = anchor_spl::token::Mint::try_deserialize_unchecked(data)?;

    Ok(account)
}
