use std::{collections::HashMap, sync::Arc};

use solana_sdk::pubkey::Pubkey;

use jet_instructions::margin_swap::SwapAccounts;
use jet_program_common::programs::{OPENBOOK, ORCA_WHIRLPOOL};

use super::AccountStates;
use crate::client::ClientResult;

mod openbook;
mod whirlpool;

/// Generalized data about a DEX pool/market
pub struct DexState {
    pub program: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub swap_a_to_b_accounts: Arc<dyn SwapAccounts + Send + Sync>,
    pub swap_b_to_a_accounts: Arc<dyn SwapAccounts + Send + Sync>,
}

/// Sync latest state for all dexes
pub async fn sync(states: &AccountStates) -> ClientResult<()> {
    let mut dex_map = HashMap::new();

    for dex in &states.config.exchanges {
        let entry = dex_map.entry(dex.program).or_insert_with(Vec::new);
        entry.push(dex.clone());
    }

    let dexes_for_program = |program| dex_map.get(program).cloned().unwrap_or(vec![]);

    whirlpool::load_orca_whirlpools(states, &dexes_for_program(&ORCA_WHIRLPOOL)).await?;
    openbook::load_openbook_markets(states, &dexes_for_program(&OPENBOOK)).await?;

    Ok(())
}

/// Sync latest state for orca whirlpools
pub async fn sync_whirlpools(states: &AccountStates) -> ClientResult<()> {
    whirlpool::load_orca_whirlpools(
        states,
        &states
            .config
            .exchanges
            .iter()
            .filter(|d| d.program == ORCA_WHIRLPOOL)
            .cloned()
            .collect::<Vec<_>>(),
    )
    .await
}
