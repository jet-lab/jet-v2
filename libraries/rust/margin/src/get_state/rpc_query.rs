use std::sync::Arc;

use anchor_lang::AccountDeserialize;
use anyhow::{bail, Result};
use jet_simulation::SolanaRpcClient;
use solana_sdk::pubkey::Pubkey;

/// todo use
/// read an account on chain as an anchor type
pub async fn get_anchor_account<T: AccountDeserialize>(
    rpc: &Arc<dyn SolanaRpcClient>,
    address: &Pubkey,
) -> Result<T> {
    let account_data = rpc.get_account(address).await?;

    match account_data {
        None => bail!("no account state found for account {}", address),
        Some(account) => Ok(T::try_deserialize(&mut &account.data[..])?),
    }
}
