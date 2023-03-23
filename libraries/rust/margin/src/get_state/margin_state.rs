use std::sync::Arc;

use anchor_lang::AccountDeserialize;
use anyhow::{bail, Result};
use jet_instructions::margin::derive_token_config;
use jet_margin::{MarginAccount, TokenConfig};
use jet_metadata::{PositionTokenMetadata, TokenMetadata};
use jet_simulation::SolanaRpcClient;
use solana_sdk::pubkey::Pubkey;

pub(crate) async fn get_position_metadata(
    rpc: &Arc<dyn SolanaRpcClient>,
    position_token_mint: &Pubkey,
) -> Result<PositionTokenMetadata> {
    let (md_address, _) =
        Pubkey::find_program_address(&[position_token_mint.as_ref()], &jet_metadata::ID);

    let account_data = rpc.get_account(&md_address).await?;

    match account_data {
        None => bail!(
            "no metadata {} found for position token {}",
            md_address,
            position_token_mint
        ),
        Some(account) => Ok(PositionTokenMetadata::try_deserialize(
            &mut &account.data[..],
        )?),
    }
}

pub(crate) async fn get_position_config(
    rpc: &Arc<dyn SolanaRpcClient>,
    airspace: &Pubkey,
    token_mint: &Pubkey,
) -> Result<Option<(Pubkey, TokenConfig)>> {
    let cfg_address = derive_token_config(airspace, token_mint);
    let account_data = rpc.get_account(&cfg_address).await?;

    match account_data {
        None => Ok(None),
        Some(account) => Ok(Some((
            cfg_address,
            TokenConfig::try_deserialize(&mut &account.data[..])?,
        ))),
    }
}

pub(crate) async fn get_token_metadata(
    rpc: &Arc<dyn SolanaRpcClient>,
    token_mint: &Pubkey,
) -> Result<TokenMetadata> {
    let (md_address, _) = Pubkey::find_program_address(&[token_mint.as_ref()], &jet_metadata::ID);
    let account_data = rpc.get_account(&md_address).await?;

    match account_data {
        None => bail!("no metadata {} found for token {}", md_address, token_mint),
        Some(account) => Ok(TokenMetadata::try_deserialize(&mut &account.data[..])?),
    }
}

/// Get the latest [MarginAccount] state
pub async fn get_margin_account(
    rpc: &Arc<dyn SolanaRpcClient>,
    address: &Pubkey,
) -> Result<MarginAccount> {
    let account_data = rpc.get_account(address).await?;

    match account_data {
        None => bail!("no margin account state found for account {}", address),
        Some(account) => Ok(MarginAccount::try_deserialize(&mut &account.data[..])?),
    }
}
