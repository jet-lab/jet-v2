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

//! The openbook swap module gets all the markets that can be used in a swap

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anchor_lang::ToAccountMetas;
use anchor_spl::dex::serum_dex::state::{gen_vault_signer_key, MarketState};
use jet_margin_swap::{accounts as ix_accounts, seeds::OPENBOOK_OPEN_ORDERS, SwapRouteIdentifier};
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use jet_solana_client::rpc::AccountFilter;
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, rent::Rent, sysvar::SysvarId};

use crate::ix_builder::SwapAccounts;

/// Accounts used for a Saber swap pool
#[derive(Debug, Clone, Copy)]
pub struct OpenBookMarket {
    /// The market address
    pub market: Pubkey,
    /// Base (coin) mint
    pub base_mint: Pubkey,
    /// Quote (price currency) mint
    pub quote_mint: Pubkey,
    ///
    pub request_queue: Pubkey,
    ///
    pub bids: Pubkey,
    ///
    pub asks: Pubkey,
    ///
    pub event_queue: Pubkey,
    ///
    pub base_vault: Pubkey,
    ///
    pub quote_vault: Pubkey,
    ///
    pub vault_signer: Pubkey,
    ///
    pub program: Pubkey,
    ///
    pub base_lot_size: u64,
    ///
    pub quote_lot_size: u64,
    /// Base decimals for price conversions
    pub base_mint_decimals: u8,
    /// Quote decimals for price conversions
    pub quote_mint_decimals: u8,
}

impl OpenBookMarket {
    /// Get all swap pools that contain pairs of supported mints
    pub async fn get_markets(
        rpc: &Arc<dyn SolanaRpcClient>,
        supported_mints: &HashSet<Pubkey>,
    ) -> anyhow::Result<HashMap<(Pubkey, Pubkey), Self>> {
        let program = anchor_spl::dex::id();
        let size = std::mem::size_of::<MarketState>();
        let accounts = rpc
            .get_program_accounts(&program, vec![AccountFilter::DataSize(size + 12)]) // Some(size)
            .await
            .unwrap();

        let mut pool_sizes = HashMap::with_capacity(accounts.len());
        for (market_address, market_account) in accounts {
            let market = bytemuck::from_bytes::<MarketState>(&market_account.data[5..(size + 5)]);
            if market.check_flags().is_err() {
                continue;
            }
            let base_mint = pubkey_from_slice(market.coin_mint);
            let quote_mint = pubkey_from_slice(market.pc_mint);

            if supported_mints
                .get(&base_mint)
                .and_then(|_| supported_mints.get(&quote_mint))
                .is_none()
            {
                continue;
            }

            let Ok(mut parsed_market) = Self::from_market_state(market_address, program, market) else {
                continue;
            };
            let Ok(base_) = super::find_mint(rpc, &parsed_market.base_mint).await else {
                continue;
            };
            let Ok(quote_) = super::find_mint(rpc, &parsed_market.quote_mint).await else {
                continue;
            };
            parsed_market.base_mint_decimals = base_.decimals;
            parsed_market.quote_mint_decimals = quote_.decimals;

            // Check if there is a pool, insert if none, replace if smaller
            pool_sizes
                .entry((base_mint, quote_mint))
                .and_modify(|(e_pool, e_value)| {
                    // Take the market with the highest quote amount as a convenience
                    if &{ market.pc_deposits_total } > e_value {
                        // Replace with current pool
                        *e_pool = parsed_market;
                        *e_value = market.pc_deposits_total;
                    }
                })
                .or_insert((parsed_market, market.pc_deposits_total));
        }
        let markets = pool_sizes
            .into_iter()
            .map(|(k, (p, _))| (k, p))
            .collect::<HashMap<_, _>>();

        Ok(markets)
    }

    #[inline]
    /// Little helper to get an [OpenBookMarket] from its on-chain rep
    fn from_market_state(
        market_address: Pubkey,
        program: Pubkey,
        market: &MarketState,
    ) -> anyhow::Result<Self> {
        let vault_signer =
            gen_vault_signer_key(market.vault_signer_nonce, &market_address, &program)?;
        Ok(Self {
            program,
            market: market_address,
            base_mint: pubkey_from_slice(market.coin_mint),
            quote_mint: pubkey_from_slice(market.pc_mint),
            request_queue: pubkey_from_slice(market.req_q),
            bids: pubkey_from_slice(market.bids),
            asks: pubkey_from_slice(market.asks),
            event_queue: pubkey_from_slice(market.event_q),
            base_vault: pubkey_from_slice(market.coin_vault),
            quote_vault: pubkey_from_slice(market.pc_vault),
            vault_signer,
            base_lot_size: market.coin_lot_size,
            quote_lot_size: market.pc_lot_size,
            base_mint_decimals: 0,
            quote_mint_decimals: 0,
        })
    }
}

#[inline]
fn pubkey_from_slice(slice: [u64; 4]) -> Pubkey {
    let address_bytes: [u8; 32] = bytemuck::cast_slice(&slice).try_into().unwrap();
    Pubkey::from(address_bytes)
}

impl SwapAccounts for OpenBookMarket {
    fn to_account_meta(&self, authority: Pubkey) -> Vec<AccountMeta> {
        let (open_orders, _) = Pubkey::find_program_address(
            &[
                OPENBOOK_OPEN_ORDERS,
                authority.as_ref(),
                self.market.as_ref(),
            ],
            &jet_margin_swap::id(),
        );

        ix_accounts::OpenbookSwapInfo {
            market: self.market,
            /// This relies on a deterministic open orders account
            open_orders,
            request_queue: self.request_queue,
            event_queue: self.event_queue,
            market_bids: self.bids,
            market_asks: self.asks,
            base_vault: self.base_vault,
            quote_vault: self.quote_vault,
            vault_signer: self.vault_signer,
            dex_program: self.program,
            rent: Rent::id(),
        }
        .to_account_metas(None)
    }

    fn pool_tokens(&self) -> (Pubkey, Pubkey) {
        (self.base_mint, self.quote_mint)
    }

    fn route_type(&self) -> SwapRouteIdentifier {
        SwapRouteIdentifier::OpenBook
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }
}
