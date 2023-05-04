use std::{collections::HashSet, sync::Arc};

use anchor_lang::{prelude::Pubkey, AccountDeserialize};
use jet_margin_sdk::{
    fixed_term::Market,
    ix_builder::MarginPoolIxBuilder,
    jet_margin::{TokenAdmin, TokenConfig},
};
use jet_simulation::SolanaRpcClient;
use solana_client::rpc_filter::RpcFilterType;
use solana_sdk::account::ReadableAccount;

use crate::Result;

#[derive(Default)]
pub struct ProgramAddresses {
    pub airspace: Pubkey,
    pub margin_pool: HashSet<Pubkey>,
    pub fixed_term: HashSet<Pubkey>,
}

impl ProgramAddresses {
    pub async fn fetch(solana_client: &Arc<dyn SolanaRpcClient>, airspace: Pubkey) -> Result<Self> {
        let mut addresses = ProgramAddresses {
            airspace,
            ..Default::default()
        };
        // Get token configs
        let token_configs =
            find_anchor_accounts::<TokenConfig>(solana_client, &jet_margin_sdk::jet_margin::id())
                .await?
                .into_iter()
                .filter(|(_, config)| config.airspace == airspace)
                .collect::<Vec<_>>();

        find_anchor_accounts::<Market>(solana_client, &jet_margin_sdk::jet_fixed_term::id())
            .await?
            .into_iter()
            .for_each(|(address, market)| {
                if market.airspace == airspace {
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

/// Find Anchor accounts for a specific type `T`
///
/// TODO: requiring a solanarpcclient seems inconvenient here
async fn find_anchor_accounts<T: AccountDeserialize>(
    rpc: &Arc<dyn SolanaRpcClient>,
    program: &Pubkey,
) -> Result<Vec<(Pubkey, T)>> {
    let result = rpc
        .get_program_accounts(
            program,
            vec![RpcFilterType::DataSize(std::mem::size_of::<T>() as u64 + 8)],
        )
        .await?;
    Ok(result
        .into_iter()
        .filter_map(|(address, account)| {
            T::try_deserialize(&mut account.data())
                .map(|t| (address, t))
                .ok()
        })
        .collect())
}
