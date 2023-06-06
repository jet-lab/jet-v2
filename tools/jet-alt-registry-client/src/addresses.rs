use std::collections::HashSet;

use anchor_lang::prelude::Pubkey;
use jet_margin_sdk::{
    fixed_term::Market,
    ix_builder::MarginPoolIxBuilder,
    jet_margin::{TokenAdmin, TokenConfig},
};
use jet_solana_client::rpc::{SolanaRpc, SolanaRpcExtra};

use crate::Result;

#[derive(Default)]
pub struct ProgramAddresses {
    pub airspace: Pubkey,
    pub margin_pool: HashSet<Pubkey>,
    pub fixed_term: HashSet<Pubkey>,
}

impl ProgramAddresses {
    pub async fn fetch<Rpc>(rpc: &Rpc, airspace: Pubkey) -> Result<Self>
    where
        Rpc: SolanaRpc + Send + Sync + 'static,
    {
        let mut addresses = ProgramAddresses {
            airspace,
            ..Default::default()
        };

        // Get token configs
        let token_configs = rpc
            .find_anchor_accounts::<TokenConfig>()
            .await?
            .into_iter()
            .filter(|(_, config)| config.airspace == airspace)
            .collect::<Vec<_>>();

        rpc.find_anchor_accounts::<Market>()
            .await?
            .into_iter()
            .for_each(|(address, market)| {
                if market.airspace == airspace {
                    // This serves as an identifier that addresses in here are fixed term
                    addresses
                        .fixed_term
                        .insert(jet_margin_sdk::jet_fixed_term::id());
                    addresses.fixed_term.insert(address);
                    addresses.fixed_term.insert(market.asks);
                    addresses.fixed_term.insert(market.bids);
                    addresses.fixed_term.insert(market.claims_mint);
                    addresses.fixed_term.insert(market.event_queue);
                    addresses.fixed_term.insert(market.fee_destination);
                    addresses.fixed_term.insert(market.fee_vault);
                    addresses.fixed_term.insert(market.orderbook_market_state);
                    addresses.fixed_term.insert(market.ticket_collateral_mint);
                    addresses.fixed_term.insert(market.ticket_mint);
                    addresses.fixed_term.insert(market.ticket_oracle);
                    addresses.fixed_term.insert(market.underlying_oracle);
                    addresses.fixed_term.insert(market.underlying_token_mint);
                    addresses.fixed_term.insert(market.underlying_token_vault);
                }
            });

        token_configs.iter().for_each(|(_, config)| {
            match config.admin {
                TokenAdmin::Adapter(p) if p == jet_margin_sdk::jet_margin_pool::ID => {
                    // Marker for margin pool addresses
                    addresses
                        .margin_pool
                        .insert(jet_margin_sdk::jet_margin_pool::id());
                    // This is duplicative, but is convenient so it's fine
                    let ix = MarginPoolIxBuilder::new(config.underlying_mint);
                    addresses.margin_pool.insert(ix.address);
                    addresses.margin_pool.insert(ix.deposit_note_mint);
                    addresses.margin_pool.insert(ix.loan_note_mint);
                    addresses.margin_pool.insert(ix.token_mint);
                    addresses.margin_pool.insert(ix.vault);
                }
                _ => {}
            };
        });

        Ok(addresses)
    }
}
