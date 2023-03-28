use std::sync::Arc;

use anchor_lang::AccountDeserialize;
use anyhow::{Context, Result};
use jet_instructions::margin::derive_token_config;
use jet_margin::{MarginAccount, TokenConfig};
use jet_metadata::{PositionTokenMetadata, TokenMetadata};
use jet_simulation::SolanaRpcClient;
use solana_sdk::pubkey::Pubkey;

use super::get_anchor_account;

pub(crate) async fn get_position_metadata(
    rpc: &Arc<dyn SolanaRpcClient>,
    position_token_mint: &Pubkey,
) -> Result<PositionTokenMetadata> {
    let (md_address, _) =
        Pubkey::find_program_address(&[position_token_mint.as_ref()], &jet_metadata::ID);

    get_anchor_account(rpc, &md_address)
        .await
        .with_context(|| format!("metadata for position token {position_token_mint}"))
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
    get_anchor_account(rpc, &md_address)
        .await
        .with_context(|| format!("metadata for token_mint {token_mint}"))
}

/// Get the latest [MarginAccount] state
pub async fn get_margin_account(
    rpc: &Arc<dyn SolanaRpcClient>,
    address: &Pubkey,
) -> Result<MarginAccount> {
    get_anchor_account(rpc, address)
        .await
        .context("margin account")
}
