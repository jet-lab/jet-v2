use std::collections::HashSet;

use jet_margin::MAX_ORACLE_STALENESS;
use jet_program_common::Number128;
use jet_solana_client::{NetworkUserInterface, NetworkUserInterfaceExt};

use super::AccountStates;
use crate::client::ClientResult;

/// The current state of an oracle that provides pricing information
pub struct PriceOracleState {
    pub price: Number128,
    pub is_valid: bool,
}

/// Sync latest state for all oracles
pub async fn sync<I: NetworkUserInterface>(states: &AccountStates<I>) -> ClientResult<I, ()> {
    let mut oracle_address_set = HashSet::new();

    oracle_address_set.extend(states.config.tokens.iter().map(|t| t.oracle));
    oracle_address_set.extend(states.cache.addresses_of::<PriceOracleState>());

    let oracles: Vec<_> = oracle_address_set.drain().collect();

    let accounts = states.network.get_accounts_all(&oracles).await?;

    for (index, account) in accounts.into_iter().enumerate() {
        let address = oracles[index];

        let mut account = match account {
            Some(account) => account,
            None => {
                log::error!("oracle {address} does not exist");
                continue;
            }
        };

        let price_feed = match pyth_sdk_solana::load_price_feed_from_account(&address, &mut account)
        {
            Ok(feed) => feed,
            Err(e) => {
                log::error!("could not parse oracle '{address}': {e}");
                continue;
            }
        };

        let current_price = price_feed.get_price_unchecked();
        let current_time = chrono::Utc::now().timestamp();

        let price = Number128::from_decimal(current_price.price, current_price.expo);
        let state = PriceOracleState {
            price,
            is_valid: price_feed
                .get_price_no_older_than(current_time, MAX_ORACLE_STALENESS as u64)
                .is_some(),
        };

        states.cache.set(&address, state);
    }

    Ok(())
}
