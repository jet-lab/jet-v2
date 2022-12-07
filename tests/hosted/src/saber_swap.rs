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

use std::collections::HashMap;
use std::sync::Arc;

use anchor_lang::prelude::Pubkey;
use anyhow::bail;
use anyhow::Error;
use anyhow::Result;
use async_trait::async_trait;
use itertools::Itertools;
use jet_margin_sdk::swap::saber_swap::SaberSwapPool;
use jet_margin_sdk::util::asynchronous::MapAsync;
use jet_simulation::{generate_keypair, solana_rpc_api::SolanaRpcClient};
use saber_client::state::SwapInfo;
use solana_sdk::signature::Signer;
use solana_sdk::{program_pack::Pack, system_instruction};
use tokio::try_join;

use crate::runtime::SolanaTestContext;
use crate::tokens::TokenManager;

pub type SwapRegistry = HashMap<Pubkey, HashMap<Pubkey, SaberSwapPool>>;

pub const ONE: u64 = 1_000_000_000;

pub async fn create_swap_pools(ctx: &SolanaTestContext, mints: &[Pubkey]) -> Result<SwapRegistry> {
    let mut registry = SwapRegistry::new();
    for (one, two, pool) in mints
        .iter()
        .combinations(2)
        .map(|c| (*c[0], *c[1]))
        .map_async(|(one, two)| create_and_insert(ctx, one, two))
        .await?
    {
        registry.entry(one).or_default().insert(two, pool);
        registry.entry(two).or_default().insert(one, pool);
    }

    Ok(registry)
}

async fn create_and_insert(
    ctx: &SolanaTestContext,
    one: Pubkey,
    two: Pubkey,
) -> Result<(Pubkey, Pubkey, SaberSwapPool)> {
    let pool =
        SaberSwapPool::configure(ctx, &one, &two, 100_000_000 * ONE, 100_000_000 * ONE).await?;

    Ok((one, two, pool))
}

#[async_trait]
pub trait SaberSwapPoolConfig: Sized {
    async fn configure(
        ctx: &SolanaTestContext,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
        a_amount: u64,
        b_amount: u64,
    ) -> Result<Self, Error>;

    async fn balances(&self, rpc: &Arc<dyn SolanaRpcClient>)
        -> Result<HashMap<Pubkey, u64>, Error>;
}

#[async_trait]
impl SaberSwapPoolConfig for SaberSwapPool {
    /// Configure a new swap pool. Supply the amount of tokens to avoid needing
    /// to deposit tokens separately.
    async fn configure(
        ctx: &SolanaTestContext,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
        a_amount: u64,
        b_amount: u64,
    ) -> Result<Self, Error> {
        // Configure the input accounts required by the pool
        let rpc = &ctx.rpc;

        // Create a TokenManager instance
        let token_manager = TokenManager::new(ctx.clone());
        let keypair = generate_keypair();

        // Token mint decimals must be the same, check them early
        let mint_a_account = token_manager.get_mint(mint_a).await?;
        let mint_b_account = token_manager.get_mint(mint_b).await?;
        if mint_a_account.decimals != mint_b_account.decimals {
            bail!("Mints must have the same decimals");
        }

        // Create an empty pool state account
        let program_id = saber_client::id();

        let space = SwapInfo::LEN;
        let rent_lamports = rpc.get_minimum_balance_for_rent_exemption(space).await?;
        let ix_pool_state_account = system_instruction::create_account(
            &rpc.payer().pubkey(),
            &keypair.pubkey(),
            rent_lamports,
            space as u64,
            &program_id,
        );

        // Pool authority
        let (pool_authority, pool_nonce) =
            Pubkey::find_program_address(&[keypair.pubkey().as_ref()], &program_id);
        // The accounts are funded to avoid having to fund them further
        let (reserve_a, reserve_b, pool_mint) = try_join!(
            // Token A account
            token_manager.create_account_funded(mint_a, &pool_authority, a_amount),
            // Token B account
            token_manager.create_account_funded(mint_b, &pool_authority, b_amount),
            // Pool token mint
            token_manager.create_token(mint_a_account.decimals, Some(&pool_authority), None),
        )?;
        let payer = rpc.payer().pubkey();
        let (admin_fee_a, admin_fee_b, lp_destination) = try_join!(
            token_manager.create_account(mint_a, &payer),
            token_manager.create_account(mint_b, &payer),
            token_manager.create_account(&pool_mint, &payer),
        )?;

        let ix_init = saber_client::instruction::initialize(
            &spl_token::id(),
            &keypair.pubkey(),
            &pool_authority,
            &payer,
            &admin_fee_a,
            &admin_fee_b,
            mint_a,
            &reserve_a,
            mint_b,
            &reserve_b,
            &pool_mint,
            &lp_destination,
            pool_nonce,
            100,
            saber_client::fees::Fees {
                admin_trade_fee_numerator: 1,
                admin_trade_fee_denominator: 400,
                admin_withdraw_fee_numerator: 2,
                admin_withdraw_fee_denominator: 500,
                trade_fee_numerator: 4,
                trade_fee_denominator: 100,
                withdraw_fee_numerator: 1,
                withdraw_fee_denominator: 100,
            },
        )?;

        // Create and send transaction
        let transaction = rpc
            .create_transaction(&[&keypair], &[ix_pool_state_account, ix_init])
            .await?;
        rpc.send_and_confirm_transaction(&transaction).await?;

        Ok(Self {
            pool: keypair.pubkey(),
            pool_authority,
            mint_a: *mint_a,
            mint_b: *mint_b,
            token_a: reserve_a,
            token_b: reserve_b,
            pool_mint,
            fee_a: admin_fee_a,
            fee_b: admin_fee_b,
            program: program_id,
        })
    }

    async fn balances(
        &self,
        rpc: &Arc<dyn SolanaRpcClient>,
    ) -> Result<HashMap<Pubkey, u64>, Error> {
        let mut mint_to_balance: HashMap<Pubkey, u64> = HashMap::new();
        mint_to_balance.insert(
            self.mint_a,
            amount(&rpc.get_account(&self.token_a).await?.unwrap().data),
        );
        mint_to_balance.insert(
            self.mint_b,
            amount(&rpc.get_account(&self.token_b).await?.unwrap().data),
        );

        Ok(mint_to_balance)
    }
}

pub fn amount(data: &[u8]) -> u64 {
    let mut amount_bytes = [0u8; 8];
    amount_bytes.copy_from_slice(&data[64..72]);
    u64::from_le_bytes(amount_bytes)
}

pub fn mint(data: &[u8]) -> Pubkey {
    let mut mint_bytes = [0u8; 32];
    mint_bytes.copy_from_slice(&data[..32]);
    Pubkey::new_from_array(mint_bytes)
}
