use solana_sdk::{
    account::ReadableAccount, address_lookup_table_account::AddressLookupTableAccount,
    pubkey::Pubkey,
};

use lookup_table_registry::RegistryAccount;

use jet_solana_client::rpc::{ClientError, SolanaRpc, SolanaRpcExtra};

/// Get all the lookup tables associated with an authority
pub async fn resolve_lookup_tables(
    rpc: &(dyn SolanaRpc + 'static),
    authority: &Pubkey,
) -> Result<Vec<AddressLookupTableAccount>, ClientError> {
    let registry_address =
        Pubkey::find_program_address(&[authority.as_ref()], &lookup_table_registry::ID).0;

    let Some(registry) = rpc.try_get_anchor_account::<RegistryAccount>(&registry_address).await? else {
        log::warn!("no registry account for authority {}", authority);
        return Ok(vec![]);
    };

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

    let accounts = rpc.get_accounts_all(&addresses).await.unwrap();
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
            Some(table)
        })
        .collect::<Vec<_>>();

    log::info!(
        "resolved {} lookup tables from authority {}",
        tables.len(),
        authority
    );

    Ok(tables)
}
