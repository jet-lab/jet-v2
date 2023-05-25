use std::sync::Arc;

use solana_sdk::pubkey::Pubkey;

use jet_instructions::margin_pool::derive_margin_pool;
use jet_margin_pool::MarginPool;
use jet_solana_client::rpc::SolanaRpcExtra;

use super::AccountStates;
use crate::client::ClientResult;

pub trait MarginPoolCacheExt {
    fn get_pool(&self, token: &Pubkey) -> Option<Arc<MarginPool>>;
}

impl MarginPoolCacheExt for AccountStates {
    fn get_pool(&self, token: &Pubkey) -> Option<Arc<MarginPool>> {
        self.get::<MarginPool>(&derive_margin_pool(&self.config.airspace, token))
    }
}

/// Sync latest state for all pools
pub async fn sync(states: &AccountStates) -> ClientResult<()> {
    let pools = states
        .config
        .tokens
        .iter()
        .map(|info| derive_margin_pool(&states.config.airspace, &info.mint))
        .collect::<Vec<_>>();

    let accounts = states
        .network
        .try_get_anchor_accounts::<MarginPool>(&pools)
        .await?;

    let time = states.get_current_time();

    for (index, account) in accounts.into_iter().enumerate() {
        let address = pools[index];

        if let Some(mut pool) = account {
            // make sure local client sees current interest
            while !pool.accrue_interest(time) {}

            states.cache.set(&address, pool);
        }
    }

    Ok(())
}
