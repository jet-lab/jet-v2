use std::collections::HashSet;

use anchor_lang::prelude::Pubkey;
use jet_program_common::Number128;
use jet_solana_client::rpc::SolanaRpcExtra;
use pyth_sdk_solana::{
    state::{load_price_account, PriceAccount, PriceStatus},
    PythError,
};
use solana_sdk::account_info::{Account, IntoAccountInfo};

use super::AccountStates;
use crate::client::ClientResult;

/// The current state of an oracle that provides pricing information
pub struct PriceOracleState {
    pub price: Number128,
    pub is_valid: bool,
}

/// Sync latest state for all oracles
pub async fn sync(states: &AccountStates) -> ClientResult<()> {
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

        let price_account = match load_price_account_from_account(&address, &mut account) {
            Ok(feed) => feed,
            Err(e) => {
                log::error!("could not parse oracle '{address}': {e}");
                continue;
            }
        };
        let current_price = price_account.to_price_feed(&address).get_price_unchecked();

        let price = Number128::from_decimal(current_price.price, current_price.expo);
        let state = PriceOracleState {
            price,
            is_valid: matches!(price_account.agg.status, PriceStatus::Trading),
        };

        states.cache.set(&address, state);
    }

    Ok(())
}

/// copy of `pyth_sdk_solana::load_price_feed_from_account` that returns one
/// step early, so we can access the PriceAccount.
fn load_price_account_from_account(
    price_key: &Pubkey,
    price_account: &mut impl Account,
) -> Result<PriceAccount, PythError> {
    let price_account_info = (price_key, price_account).into_account_info();
    let data = price_account_info
        .try_borrow_data()
        .map_err(|_| PythError::InvalidAccountData)?;
    load_price_account(*data).cloned()
}
