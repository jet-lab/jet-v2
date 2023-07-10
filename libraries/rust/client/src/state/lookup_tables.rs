use jet_environment::lookup_tables::resolve_lookup_tables;
use jet_margin::MarginAccount;

use crate::{state::AccountStates, ClientResult};

use super::LookupTableCache;

/// Sync latest state for all token accounts
pub async fn sync(states: &AccountStates) -> ClientResult<()> {
    // Get the airspace authority registry
    if let Some(airspace_authority) = states.config.airspace_lookup_registry_authority {
        let tables = resolve_lookup_tables(states.network.as_ref(), &airspace_authority).await?;

        if tables.is_empty() {
            log::debug!("missing lookup tables for airspace authority {airspace_authority}")
        } else {
            states
                .lookup_tables
                .set(LookupTableCache::DEFAULT_PRIORITY, tables);
        }
    }

    // Get the margin account registries
    for margin_account in states.addresses_of::<MarginAccount>() {
        let tables = resolve_lookup_tables(states.network.as_ref(), &margin_account).await?;

        if tables.is_empty() {
            log::debug!("missing lookup tables for margin account {margin_account}");
        } else {
            states
                .lookup_tables
                .set(LookupTableCache::DEFAULT_PRIORITY, tables);
        }
    }

    Ok(())
}
