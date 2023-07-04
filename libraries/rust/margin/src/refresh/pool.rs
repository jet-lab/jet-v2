//! Refresh margin deposits and pool positions.

use anyhow::Result;

use jet_instructions::{margin::accounting_invoke, margin_pool::MarginPoolIxBuilder};
use jet_margin::MarginAccount;
use jet_simulation::SolanaRpcClient;
use jet_solana_client::transaction::TransactionBuilder;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, sync::Arc};

use crate::{
    get_state::{get_position_metadata, get_token_metadata},
    margin_account_ext::MarginAccountExt,
};

use super::position_refresher::define_refresher;

define_refresher!(PoolRefresher, refresh_all_pool_positions);

/// Identify all pool positions, find metadata, and refresh them.
pub async fn refresh_all_pool_positions(
    rpc: &Arc<dyn SolanaRpcClient>,
    state: &MarginAccount,
) -> Result<Vec<TransactionBuilder>> {
    Ok(refresh_all_pool_positions_underlying_to_tx(rpc, state)
        .await?
        .into_values()
        .collect())
}

/// Identify all pool positions, find metadata, and refresh them.   
/// Map keyed by underlying token mint.
pub async fn refresh_all_pool_positions_underlying_to_tx(
    rpc: &Arc<dyn SolanaRpcClient>,
    state: &MarginAccount,
) -> Result<HashMap<Pubkey, TransactionBuilder>> {
    let mut txns = HashMap::new();
    let address = state.address();
    for position in state.positions() {
        if position.adapter != jet_margin_pool::ID {
            continue;
        }
        let p_metadata = get_position_metadata(rpc, &position.token).await?;
        if txns.contains_key(&p_metadata.underlying_token_mint) {
            continue;
        }
        let t_metadata = get_token_metadata(rpc, &p_metadata.underlying_token_mint).await?;
        let ix_builder = MarginPoolIxBuilder::new(p_metadata.underlying_token_mint);
        let inner = ix_builder.margin_refresh_position(address, t_metadata.pyth_price);
        let ix = accounting_invoke(state.airspace, address, inner);

        txns.insert(p_metadata.underlying_token_mint, ix.into());
    }

    Ok(txns)
}
