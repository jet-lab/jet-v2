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

use std::sync::Arc;

use anchor_lang::Id;
use anchor_spl::dex::{serum_dex, Dex};
use jet_margin_sdk::ix_builder::SerumMarketV3;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use jet_simulation::{generate_keypair, send_and_confirm};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use solana_sdk::system_instruction;

use crate::tokens::TokenManager;

/// Utility for creating a Serum market, and submitting transactions that don't
/// require a margin account.
pub struct SerumClient {
    rpc: Arc<dyn SolanaRpcClient>,
    market: SerumMarketV3,
}

impl SerumClient {
    /// Create a new Serum DEX market for testing, and configure [SerumMarketInfo]
    pub async fn create_market(
        rpc: Arc<dyn SolanaRpcClient>,
        base_mint: Pubkey,
        quote_mint: Pubkey,
        base_lot_size: u64,
        quote_lot_size: u64,
    ) -> anyhow::Result<Self> {
        // Initialize a Serum market
        let token_manager = TokenManager::new(rpc.clone());

        let market = generate_keypair();
        let market_size = std::mem::size_of::<serum_dex::state::MarketState>() + 12;
        let market_lamports = rpc
            .get_minimum_balance_for_rent_exemption(market_size)
            .await?;
        let market_ix = system_instruction::create_account(
            &rpc.payer().pubkey(),
            &market.pubkey(),
            market_lamports,
            market_size as u64,
            &Dex::id(),
        );

        let (vault_nonce, vault_signer) = {
            let mut i = 0;
            loop {
                assert!(i < 100);
                if let Ok(pk) =
                    serum_dex::state::gen_vault_signer_key(i, &market.pubkey(), &Dex::id())
                {
                    break (i, pk);
                }
                i += 1;
            }
        };

        // State accounts
        let bid_ask_size = 65536 + 12;
        let bid_ask_lamports = rpc
            .get_minimum_balance_for_rent_exemption(bid_ask_size)
            .await?;
        let bids = generate_keypair();
        let asks = generate_keypair();
        let bids_ix = system_instruction::create_account(
            &rpc.payer().pubkey(),
            &bids.pubkey(),
            bid_ask_lamports,
            bid_ask_size as u64,
            &Dex::id(),
        );
        let asks_ix = system_instruction::create_account(
            &rpc.payer().pubkey(),
            &asks.pubkey(),
            bid_ask_lamports,
            bid_ask_size as u64,
            &Dex::id(),
        );

        let event_queue_size = 262144 + 12;
        let request_queue_size = 5120 + 12;
        let events_lamports = rpc
            .get_minimum_balance_for_rent_exemption(event_queue_size)
            .await?;
        let requests_lamports = rpc
            .get_minimum_balance_for_rent_exemption(request_queue_size)
            .await?;
        let events = generate_keypair();
        let requests = generate_keypair();
        let events_ix = system_instruction::create_account(
            &rpc.payer().pubkey(),
            &events.pubkey(),
            events_lamports,
            event_queue_size as u64,
            &Dex::id(),
        );
        let requests_ix = system_instruction::create_account(
            &rpc.payer().pubkey(),
            &requests.pubkey(),
            requests_lamports,
            request_queue_size as u64,
            &Dex::id(),
        );

        // Split transactions up
        send_and_confirm(
            &rpc,
            &[market_ix, bids_ix, asks_ix, events_ix, requests_ix],
            &[&market, &bids, &asks, &events, &requests],
        )
        .await?;

        let base_vault = token_manager
            .create_account(&base_mint, &vault_signer)
            .await?;
        let quote_vault = token_manager
            .create_account(&quote_mint, &vault_signer)
            .await?;

        let init_ix = serum_dex::instruction::initialize_market(
            &market.pubkey(),
            &Dex::id(),
            &base_mint,
            &quote_mint,
            &base_vault,
            &quote_vault,
            None,
            None,
            &bids.pubkey(),
            &asks.pubkey(),
            &requests.pubkey(),
            &events.pubkey(),
            base_lot_size,
            quote_lot_size,
            vault_nonce,
            1,
        )?;

        send_and_confirm(&rpc, &[init_ix], &[]).await?;

        let market = SerumMarketV3 {
            market: market.pubkey(),
            bids: bids.pubkey(),
            asks: asks.pubkey(),
            request_queue: requests.pubkey(),
            event_queue: events.pubkey(),
            base_token: base_mint,
            quote_token: quote_mint,
            base_vault,
            quote_vault,
            vault_signer,
        };

        // This was to register the market through the adapter so it can be
        // traded with, but we don't need this yet.
        // Self::register_market_info(&rpc, &market).await?;

        Ok(Self { rpc, market })
    }

    pub fn market(&self) -> &SerumMarketV3 {
        &self.market
    }

    pub async fn match_orders(
        &self,
        base_fee_receivable: Pubkey,
        quote_fee_receivable: Pubkey,
        limit: u16,
    ) -> anyhow::Result<()> {
        let instruction = serum_dex::instruction::match_orders(
            &Dex::id(),
            &self.market.market,
            &self.market.request_queue,
            &self.market.bids,
            &self.market.asks,
            &self.market.event_queue,
            &base_fee_receivable,
            &quote_fee_receivable,
            limit,
        )?;

        send_and_confirm(&self.rpc, &[instruction], &[]).await?;

        Ok(())
    }

    pub async fn consume_events(
        &self,
        base_fee_receivable: Pubkey,
        quote_fee_receivable: Pubkey,
        open_order_accounts: Vec<&Pubkey>,
        limit: u16,
    ) -> anyhow::Result<()> {
        let instruction = serum_dex::instruction::consume_events(
            &Dex::id(),
            open_order_accounts,
            &self.market.market,
            &self.market.event_queue,
            &base_fee_receivable,
            &quote_fee_receivable,
            limit,
        )?;

        send_and_confirm(&self.rpc, &[instruction], &[]).await?;

        Ok(())
    }
}
