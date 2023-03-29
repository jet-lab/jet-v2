//! Refresh margin deposits and pool positions.

use anyhow::Result;
use jet_instructions::margin::{derive_margin_account_from_state, refresh_deposit_position};
use jet_margin::{MarginAccount, TokenOracle};
use jet_simulation::SolanaRpcClient;
use jet_solana_client::transaction::TransactionBuilder;
use std::sync::Arc;

use crate::get_state::get_position_config;

use super::position_refresher::define_refresher;

define_refresher!(DepositRefresher, refresh_deposit_positions);

/// Refresh direct ATA deposit positions managed by the margin program
pub async fn refresh_deposit_positions(
    rpc: &Arc<dyn SolanaRpcClient>,
    state: &MarginAccount,
) -> Result<Vec<TransactionBuilder>> {
    let mut instructions = vec![];
    let address = derive_margin_account_from_state(state);
    for position in state.positions() {
        let (_, p_config) = match get_position_config(rpc, &state.airspace, &position.token).await?
        {
            None => continue,
            Some(r) => r,
        };

        if position.token != p_config.underlying_mint {
            continue;
        }

        let token_oracle = match p_config.oracle().unwrap() {
            TokenOracle::Pyth { price, .. } => price,
        };

        let refresh =
            refresh_deposit_position(&state.airspace, address, position.token, token_oracle, true);
        instructions.push(refresh.into());
    }

    Ok(instructions)
}
