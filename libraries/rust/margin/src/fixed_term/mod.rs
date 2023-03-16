#![allow(missing_docs)]

pub mod error;
pub mod event_consumer;
mod ix_builder;
pub mod settler;

pub use ix_builder::*;

use anchor_lang::AccountDeserialize;
use jet_simulation::SolanaRpcClient;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, sync::Arc};

/// Find all the fixed term markets.
/// TODO: generalize this to rpc client, code from liquidator may be useful
pub async fn find_markets(
    rpc: &Arc<dyn SolanaRpcClient>,
) -> anyhow::Result<HashMap<Pubkey, Market>> {
    Ok(rpc
        .get_program_accounts(&jet_fixed_term::ID, Some(std::mem::size_of::<Market>() + 8))
        .await?
        .into_iter()
        .flat_map(|(k, account)| {
            match Market::try_deserialize(&mut &account.data[..]).map_err(anyhow::Error::from) {
                Ok(mkt) => Some((k, mkt)),
                Err(_) => None,
            }
        })
        .collect())
}
