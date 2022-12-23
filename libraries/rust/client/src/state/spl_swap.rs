use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token_swap::state::SwapV1;

use super::{tokens::TokenAccount, AccountStates};
use crate::{
    client::ClientResult, programs::ORCA_V2, ClientError, ClientInterfaceExt, UserNetworkInterface,
};

/// Sync latest state for all swap pools
pub async fn sync<I: UserNetworkInterface>(states: &AccountStates<I>) -> ClientResult<I, ()> {
    let swap_programs = [ORCA_V2];
    let addresses = states
        .config
        .exchanges
        .iter()
        .filter_map(|dex| {
            swap_programs
                .iter()
                .any(|id| *id == dex.program)
                .then_some(dex.address)
        })
        .collect::<Vec<_>>();

    load(states, &addresses).await
}

/// Load state for given swap pools
async fn load<I: UserNetworkInterface>(
    states: &AccountStates<I>,
    addresses: &[Pubkey],
) -> ClientResult<I, ()> {
    let accounts = states.network.get_accounts_all(addresses).await?;

    for (address, maybe_account) in addresses.iter().zip(accounts) {
        if let Some(account) = maybe_account {
            let data = SwapV1::unpack(&account.data[1..])
                .map_err(|e| ClientError::Deserialize(Box::new(e)))?;

            states.cache.register::<TokenAccount>(&data.token_a);
            states.cache.register::<TokenAccount>(&data.token_b);
            states.cache.set(address, data);
        }
    }

    Ok(())
}
