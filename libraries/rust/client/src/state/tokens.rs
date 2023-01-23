use std::collections::HashSet;

use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address;

use super::AccountStates;
use crate::{client::ClientResult, ClientError, ClientInterfaceExt, UserNetworkInterface};

pub type TokenAccount = spl_token::state::Account;
pub type Mint = spl_token::state::Mint;

/// Sync latest state for all token accounts
pub async fn sync<I: UserNetworkInterface>(states: &AccountStates<I>) -> ClientResult<I, ()> {
    sync_mints(states).await?;
    sync_accounts(states).await?;

    Ok(())
}

/// Sync all the mints
pub async fn sync_mints<I: UserNetworkInterface>(states: &AccountStates<I>) -> ClientResult<I, ()> {
    let mut address_set: HashSet<_> =
        HashSet::from_iter(states.config.tokens.iter().map(|t| t.mint));

    address_set.extend(states.cache.addresses_of::<Mint>());

    let addresses: Vec<_> = address_set.drain().collect();

    let accounts = states.network.get_accounts_all(&addresses).await?;

    for (address, maybe_account) in addresses.into_iter().zip(accounts) {
        if let Some(account) = maybe_account {
            let data = match Mint::unpack(&account.data) {
                Ok(data) => data,
                Err(e) => {
                    log::error!("could not parse mint {address}: {e}");
                    continue;
                }
            };

            states.cache.set(&address, data);
        }
    }

    Ok(())
}

/// Sync all the previously loaded token accounts
pub async fn sync_accounts<I: UserNetworkInterface>(
    states: &AccountStates<I>,
) -> ClientResult<I, ()> {
    let mut address_set = HashSet::new();

    address_set.extend(states.cache.addresses_of::<TokenAccount>());

    // include any relevant accounts for the user wallet
    address_set.extend(
        states
            .config
            .tokens
            .iter()
            .map(|info| get_associated_token_address(&states.network.signer(), &info.mint)),
    );

    let addresses = address_set.drain().collect::<Vec<_>>();
    load_accounts(states, &addresses).await
}

/// Load token accounts into the state cache
pub async fn load_accounts<I: UserNetworkInterface>(
    states: &AccountStates<I>,
    addresses: &[Pubkey],
) -> ClientResult<I, ()> {
    let accounts = states.network.get_accounts_all(addresses).await?;

    for (address, maybe_account) in addresses.iter().zip(accounts) {
        if let Some(account) = maybe_account {
            let data = TokenAccount::unpack(&account.data).map_err(|e| {
                eprintln!("{account:?}");
                ClientError::Deserialize(Box::new(e))
            })?;

            states.cache.set(address, data);
        }
    }

    Ok(())
}
