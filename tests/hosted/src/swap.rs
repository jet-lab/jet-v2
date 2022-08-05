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

use anchor_lang::prelude::Pubkey;
use anyhow::Error;
use anyhow::Result;
use async_trait::async_trait;
use jet_margin_sdk::solana::transaction::{SendTransactionBuilder, TransactionBuilder};
use jet_margin_sdk::spl_swap::SwapPool;
use jet_simulation::{generate_keypair, solana_rpc_api::SolanaRpcClient};
use jet_static_program_registry::{
    orca_swap_v1, orca_swap_v2, related_programs, spl_token_swap_v2,
};
use solana_sdk::signature::Signer;
use solana_sdk::{program_pack::Pack, system_instruction};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::try_join;

use crate::tokens::TokenManager;

// register swap programs
related_programs! {
    SwapProgram {[
        spl_token_swap_v2::Spl2,
        orca_swap_v1::OrcaV1,
        orca_swap_v2::OrcaV2,
    ]}
}

pub type SwapRegistry = HashMap<Pubkey, HashMap<Pubkey, SwapPool>>;

pub const ONE: u64 = 1_000_000_000;

pub async fn create_swap_pools(
    rpc: &Arc<dyn SolanaRpcClient>,
    mints: &[Pubkey],
) -> Result<SwapRegistry> {
    let mut registry = SwapRegistry::new();
    for (one, two, pool) in mints
        .iter()
        .combinations(2)
        .map(|c| (*c[0], *c[1]))
        .map_async(|(one, two)| create_and_insert(rpc, one, two))
        .await?
    {
        registry.entry(one).or_default().insert(two, pool);
        registry.entry(two).or_default().insert(one, pool);
    }

    Ok(registry)
}

async fn create_and_insert(
    rpc: &Arc<dyn SolanaRpcClient>,
    one: Pubkey,
    two: Pubkey,
) -> Result<(Pubkey, Pubkey, SwapPool)> {
    let pool = SwapPool::configure(
        rpc,
        &orca_swap_v2::id(),
        &one,
        &two,
        100_000_000 * ONE,
        100_000_000 * ONE,
    )
    .await?;

    Ok((one, two, pool))
}

#[async_trait]
pub trait SwapPoolConfig: Sized {
    async fn configure(
        rpc: &Arc<dyn SolanaRpcClient>,
        program_id: &Pubkey,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
        a_amount: u64,
        b_amount: u64,
    ) -> Result<Self, Error>;

    async fn balances(&self, rpc: &Arc<dyn SolanaRpcClient>)
        -> Result<HashMap<Pubkey, u64>, Error>;

    async fn swap<S: Signer + Sync + Send>(
        &self,
        rpc: &Arc<dyn SolanaRpcClient>,
        source: &Pubkey,
        dest: &Pubkey,
        amount_in: u64,
        signer: &S,
    ) -> Result<(), Error>;

    async fn swap_tx<S: Signer + Sync + Send>(
        &self,
        rpc: &Arc<dyn SolanaRpcClient>,
        source: &Pubkey,
        dest: &Pubkey,
        amount_in: u64,
        signer: &S,
    ) -> Result<TransactionBuilder, Error>;
}

#[async_trait]
impl SwapPoolConfig for SwapPool {
    /// Configure a new swap pool. Supply the amount of tokens to avoid needing
    /// to deposit tokens separately.
    async fn configure(
        rpc: &Arc<dyn SolanaRpcClient>,
        program_id: &Pubkey,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
        a_amount: u64,
        b_amount: u64,
    ) -> Result<Self, Error> {
        // Configure the input accounts required by the pool
        // https://spl.solana.com/token-swap#creating-a-new-token-swap-pool

        // Create a TokenManager instance
        let token_manager = TokenManager::new(rpc.clone());
        let keypair = generate_keypair();

        // Create an empty pool state account
        // The SPL Token Swap program requires extra padding of 1 byte

        let space = use_client!(*program_id, { client::state::SwapV1::LEN + 1 }).unwrap();
        let rent_lamports = rpc.get_minimum_balance_for_rent_exemption(space).await?;
        let ix_pool_state_account = system_instruction::create_account(
            &rpc.payer().pubkey(),
            &keypair.pubkey(),
            rent_lamports,
            space as u64,
            program_id,
        );

        // Pool authority
        let (pool_authority, pool_nonce) =
            Pubkey::find_program_address(&[keypair.pubkey().as_ref()], program_id);
        // The accounts are funded to avoid having to fund them further
        let (token_a, token_b, pool_mint) = try_join!(
            // Token A account
            token_manager.create_account_funded(mint_a, &pool_authority, a_amount),
            // Token B account
            token_manager.create_account_funded(mint_b, &pool_authority, b_amount),
            // Pool token mint
            token_manager.create_token(6, Some(&pool_authority), None),
        )?;
        let payer = rpc.payer().pubkey();
        let (token_fee, token_recipient) = try_join!(
            // Pool token fee account
            token_manager.create_account(&pool_mint, &payer),
            // Pool token recipient account
            token_manager.create_account(&pool_mint, &payer),
        )?;

        let ix_init = use_client!(*program_id, {
            client::instruction::initialize(
                program_id,
                &spl_token::id(),
                &keypair.pubkey(),
                &pool_authority,
                &token_a,
                &token_b,
                &pool_mint,
                &token_fee,
                &token_recipient,
                pool_nonce,
                client::curve::fees::Fees {
                    // The fee parameters are taken from one of spl-token-swap tests
                    trade_fee_numerator: 1,
                    trade_fee_denominator: 400,
                    owner_trade_fee_numerator: 2,
                    owner_trade_fee_denominator: 500,
                    owner_withdraw_fee_numerator: 4,
                    owner_withdraw_fee_denominator: 100,
                    host_fee_numerator: 1,
                    host_fee_denominator: 100,
                },
                client::curve::base::SwapCurve {
                    curve_type: client::curve::base::CurveType::ConstantProduct,
                    calculator: Box::new(client::curve::constant_product::ConstantProductCurve),
                },
            )?
        })
        .unwrap();

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
            token_a,
            token_b,
            pool_mint,
            fee_account: token_fee,
            program: *program_id,
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

    async fn swap_tx<S: Signer + Sync + Send>(
        &self,
        rpc: &Arc<dyn SolanaRpcClient>,
        source: &Pubkey,
        dest: &Pubkey,
        amount_in: u64,
        signer: &S,
    ) -> Result<TransactionBuilder, Error> {
        if amount_in == 0 {
            return Ok(TransactionBuilder::default());
        }
        let source_data = rpc.get_account(source).await?.unwrap().data;
        let dest_data = rpc.get_account(dest).await?.unwrap().data;
        let (swap_source, swap_dest) =
            if mint(&source_data) == self.mint_a && mint(&dest_data) == self.mint_b {
                (self.token_a, self.token_b)
            } else if mint(&source_data) == self.mint_b && mint(&dest_data) == self.mint_a {
                (self.token_b, self.token_a)
            } else {
                panic!("wrong pool");
            };
        let swap_ix = use_client!(self.program, {
            client::instruction::swap(
                &self.program,
                &spl_token::id(),
                &self.pool,
                &self.pool_authority,
                &signer.pubkey(),
                source,
                &swap_source,
                &swap_dest,
                dest,
                &self.pool_mint,
                &self.fee_account,
                None,
                client::instruction::Swap {
                    amount_in,
                    minimum_amount_out: 0,
                },
            )?
        })
        .unwrap();

        Ok(TransactionBuilder {
            instructions: vec![swap_ix],
            signers: vec![],
        })
    }

    async fn swap<S: Signer + Sync + Send>(
        &self,
        rpc: &Arc<dyn SolanaRpcClient>,
        source: &Pubkey,
        dest: &Pubkey,
        amount_in: u64,
        signer: &S,
    ) -> Result<(), Error> {
        let tx = self.swap_tx(rpc, source, dest, amount_in, signer).await?;
        rpc.send_and_confirm(tx).await?;

        Ok(())
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
