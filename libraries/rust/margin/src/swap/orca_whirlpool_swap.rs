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

//! The orca whirlpool swap module gets all swap pools that contain pairs of supported mints

use std::{
  collections::{HashMap, HashSet},
  sync::Arc,
};

use anchor_lang::ToAccountMetas;
use anyhow::{bail, Result};
use jet_margin_swap::{accounts as ix_accounts, SwapRouteIdentifier};
use jet_proto_math::number_128::Number128;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
// use saber_client::state::SwapInfo;
use orca_whirlpool_swap::state::SwapInfo;
use solana_sdk::{instruction::AccountMeta, program_pack::Pack, pubkey::Pubkey};

use crate::ix_builder::SwapAccounts;

use super::{find_mint, find_token};

// todo fixme
#[derive(Debug, Clone, Copy)]
pub struct OrcaWhirlpoolSwapPool {
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
  /// The account that receives fees for token A
  pub fee_a: Pubkey,
  /// The account that receives fees for token B
  pub fee_b: Pubkey,
  /// The program of the pool, to distinguish between various supported pools
  pub program: Pubkey,
}

impl OrcaWhirlpoolSwapPool {
  /// Get all swap pools that contain pairs of supported mints
  pub async fn get_pools(
      rpc: &Arc<dyn SolanaRpcClient>,
      supported_mints: &HashSet<Pubkey>,
  ) -> anyhow::Result<HashMap<(Pubkey, Pubkey), Self>> {
      let swap_program = orca_whirlpool_swap::id();
      let size = SwapInfo::LEN;
      let accounts = rpc
          .get_program_accounts(&swap_program, Some(size)) // Some(size)
          .await
          .unwrap();

      let mut pool_sizes = HashMap::with_capacity(accounts.len());
      for (swap_address, pool_account) in accounts {
          let swap = SwapInfo::unpack(&pool_account.data);
          let swap = match swap {
              Ok(swap) => swap,
              Err(_) => continue,
          };

          if !swap.is_initialized || swap.is_paused {
              continue;
          }

          let (mint_a, mint_b) = match (
              supported_mints.get(&swap.token_a.mint),
              supported_mints.get(&swap.token_b.mint),
          ) {
              (Some(a), Some(b)) => (a, b),
              _ => continue,
          };

          // Get the token balances of both sides
          let token_a = match find_token(rpc, &swap.token_a.reserves).await {
              Ok(val) => val,
              Err(_) => {
                  continue;
              }
          };
          let token_b = match find_token(rpc, &swap.token_b.reserves).await {
              Ok(val) => val,
              Err(_) => {
                  continue;
              }
          };

          let mint_a_info = find_mint(rpc, mint_a).await?;
          let mint_b_info = find_mint(rpc, mint_b).await?;

          let token_a_balance =
              Number128::from_decimal(token_a.amount, -(mint_a_info.decimals as i32));
          let token_b_balance =
              Number128::from_decimal(token_b.amount, -(mint_b_info.decimals as i32));

          let total_token = token_a_balance * token_b_balance;

          if !swap.is_initialized {
              continue;
          }

          // Check if there is a pool, insert if none, replace if smaller
          pool_sizes
              .entry((swap.token_a.mint, swap.token_b.mint))
              .and_modify(|(e_pool, e_value)| {
                  if &total_token > e_value {
                      // Replace with current pool
                      *e_pool = Self::from_saber_swap(swap_address, swap_program, &swap);
                      *e_value = total_token;
                  }
              })
              .or_insert((
                  Self::from_saber_swap(swap_address, swap_program, &swap),
                  total_token,
              ));
      }
      let swap_pools = pool_sizes
          .into_iter()
          .map(|(k, (p, _))| (k, p))
          .collect::<HashMap<_, _>>();

      Ok(swap_pools)
  }

  // todo fixme
  #[inline]
  /// Little helper to get a [SaberSwapPool] from its on-chain rep
  fn from_orca_swap(swap_address: Pubkey, swap_program: Pubkey, swap: &SwapInfo) -> Self {
      OrcaWhirlpoolSwapPool {
          pool: swap_address,
          pool_authority: Pubkey::find_program_address(
              &[swap_address.as_ref(), &[swap.nonce]],
              &swap_program,
          )
          .0,
          mint_a: swap.token_a.mint,
          mint_b: swap.token_b.mint,
          token_a: swap.token_a.reserves,
          token_b: swap.token_b.reserves,
          fee_a: swap.token_a.admin_fees,
          fee_b: swap.token_b.admin_fees,
          pool_mint: swap.pool_mint,
          program: swap_program,
      }
  }
}

// todo fixme
impl SwapAccounts for OrcaWhirlpoolSwapPool {
  fn to_account_meta(&self, src_token: &Pubkey) -> Result<Vec<AccountMeta>> {
      let (vault_from, vault_into, fee_account) = if src_token == &self.mint_a {
          (self.token_b, self.token_a, self.fee_b)
      } else if src_token == &self.mint_b {
          (self.token_a, self.token_b, self.fee_a)
      } else {
          bail!("Invalid source token")
      };
      let (swap_authority, _) =
          Pubkey::find_program_address(&[self.pool.as_ref()], &self.program);
      let accounts = ix_accounts::OrcaWhirlpoolSwapInfo {
          swap_pool: self.pool,
          authority: swap_authority,
          vault_into,
          vault_from,
          token_mint: self.pool_mint,
          admin_fee_destination: fee_account,
          swap_program: self.program,
      }
      .to_account_metas(None);

      Ok(accounts)
  }

  fn dst_token(&self, src_token: &Pubkey) -> Result<Pubkey> {
      let dst_token = if src_token == &self.mint_a {
          self.mint_b
      } else if src_token == &self.mint_b {
          self.mint_a
      } else {
          bail!("Invalid source token")
      };
      Ok(dst_token)
  }

  fn route_type(&self) -> SwapRouteIdentifier {
      SwapRouteIdentifier::OrcaWhirlpool
  }
}
