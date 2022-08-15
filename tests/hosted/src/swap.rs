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

use std::sync::Arc;

use anchor_lang::prelude::Pubkey;
use anyhow::Error;
use async_trait::async_trait;
use jet_margin_sdk::spl_swap::SplSwapPool;
use jet_simulation::{generate_keypair, solana_rpc_api::SolanaRpcClient};
use jet_static_program_registry::{
    orca_swap_v1, orca_swap_v2, related_programs, spl_token_swap_v2,
};
use solana_sdk::{program_pack::Pack, signer::Signer, system_instruction};

use crate::tokens::TokenManager;

// register swap programs
related_programs! {
    SwapProgram {[
        spl_token_swap_v2::Spl2,
        orca_swap_v1::OrcaV1,
        orca_swap_v2::OrcaV2,
    ]}
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
}

#[async_trait]
impl SwapPoolConfig for SplSwapPool {
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
        // Token A account
        // The accounts are funded to avoid having to fund them further
        let token_a = token_manager
            .create_account_funded(mint_a, &pool_authority, a_amount)
            .await?;
        // Token B account
        let token_b = token_manager
            .create_account_funded(mint_b, &pool_authority, b_amount)
            .await?;
        // Pool token mint
        let pool_mint = token_manager
            .create_token(6, Some(&pool_authority), None)
            .await?;
        // Pool token fee account
        let token_fee = token_manager
            .create_account(&pool_mint, &rpc.payer().pubkey())
            .await?;
        // Pool token recipient account
        let token_recipient = token_manager
            .create_account(&pool_mint, &rpc.payer().pubkey())
            .await?;

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
}
