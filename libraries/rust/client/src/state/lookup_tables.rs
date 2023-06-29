use std::collections::HashMap;

use anchor_lang::AccountDeserialize;
use jet_margin::MarginAccount;
use jet_solana_client::rpc::SolanaRpcExtra;
use lookup_table_registry::RegistryAccount;
use lookup_table_registry_client::LOOKUP_TABLE_REGISTRY_ID;
use solana_sdk::{
    account::ReadableAccount, address_lookup_table_account::AddressLookupTableAccount,
    pubkey::Pubkey,
};

use crate::{state::AccountStates, ClientResult};

/// Sync latest state for all token accounts
pub async fn sync(states: &AccountStates) -> ClientResult<()> {
    // Get the airspace authority registry
    if let Some(airspace_authority) = states.config.airspace_lookup_registry_authority {
        match get_lookup_tables(states, &airspace_authority).await? {
            None => {
                log::debug!("missing lookup tables for airspace authority {airspace_authority}")
            }
            Some(lookup_tables) => states.lookup_tables.set(&airspace_authority, lookup_tables),
        }
    }

    // Get the margin account registries
    for margin_account in states.addresses_of::<MarginAccount>() {
        match get_lookup_tables(states, &margin_account).await? {
            None => log::debug!("missing lookup tables for margin account {margin_account}"),
            Some(lookup_tables) => states.lookup_tables.set(&margin_account, lookup_tables),
        }
    }

    Ok(())
}

async fn get_lookup_tables(
    states: &AccountStates,
    authority: &Pubkey,
) -> ClientResult<Option<HashMap<Pubkey, AddressLookupTableAccount>>> {
    let registry_address =
        Pubkey::find_program_address(&[authority.as_ref()], &LOOKUP_TABLE_REGISTRY_ID).0;
    if let Some(registry) = states.network.get_account(&registry_address).await? {
        let registry = RegistryAccount::try_deserialize(&mut registry.data()).unwrap();

        // Get the lookup tables
        let addresses = registry
            .tables
            .iter()
            .filter_map(|entry| {
                if entry.discriminator > 1 {
                    Some(entry.table)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let accounts = states.network.get_accounts_all(&addresses).await.unwrap();
        let tables = accounts
            .into_iter()
            .zip(addresses)
            .filter_map(|(account, address)| {
                let account = account?;
                let table =
                    solana_address_lookup_table_program::state::AddressLookupTable::deserialize(
                        account.data(),
                    )
                    .ok()?;
                let table = AddressLookupTableAccount {
                    key: address,
                    addresses: table.addresses.to_vec(),
                };
                Some((address, table))
            })
            .collect::<HashMap<_, _>>();

        return Ok(Some(tables));
    }

    Ok(None)
}
