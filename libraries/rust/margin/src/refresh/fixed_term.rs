use std::sync::Arc;

use anyhow::Result;
use jet_margin::MarginAccount;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::signer::Signer;

use crate::{
    fixed_term::{find_markets, FixedTermIxBuilder},
    ix_builder::accounting_invoke,
    margin_account_ext::MarginAccountExt,
    solana::transaction::TransactionBuilder,
};

use super::position_refresher::define_refresher;

define_refresher!(FixedTermRefresher, refresh_fixed_term_positions);

/// Refreshes all fixed term positions for a margin account
pub async fn refresh_fixed_term_positions(
    rpc: &Arc<dyn SolanaRpcClient>,
    margin_account: &MarginAccount,
) -> Result<Vec<TransactionBuilder>> {
    let mut ret = vec![];
    let markets = find_markets(rpc).await?;
    let address = margin_account.address();
    for market in markets.values() {
        let bldr = FixedTermIxBuilder::new_from_state(rpc.payer().pubkey(), market);
        for position in margin_account
            .positions()
            .filter(|p| p.adapter == jet_fixed_term::id())
        {
            if position.token == bldr.claims() || position.token == bldr.collateral() {
                ret.push(
                    accounting_invoke(
                        bldr.airspace(),
                        address,
                        bldr.refresh_position(address, false),
                    )
                    .into(),
                )
            }
        }
    }

    Ok(ret)
}
