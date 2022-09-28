use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use serum_dex::state::Market;
use solana_sdk::{account_info::IntoAccountInfo, pubkey::Pubkey};

use crate::client::Client;

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerumMarketAccount {
    #[serde_as(as = "DisplayFromStr")]
    pub base_mint: Pubkey,
    #[serde_as(as = "DisplayFromStr")]
    pub quote_mint: Pubkey,
    #[serde_as(as = "DisplayFromStr")]
    pub base_vault: Pubkey,
    #[serde_as(as = "DisplayFromStr")]
    pub quote_vault: Pubkey,
    #[serde_as(as = "DisplayFromStr")]
    pub request_queue: Pubkey,
    #[serde_as(as = "DisplayFromStr")]
    pub event_queue: Pubkey,
    #[serde_as(as = "DisplayFromStr")]
    pub bids: Pubkey,
    #[serde_as(as = "DisplayFromStr")]
    pub asks: Pubkey,
    pub quote_dust_threshold: u64,
    pub base_lot_size: u64,
    pub quote_lot_size: u64,
    pub fee_rate_bps: u64,
}

pub async fn read_market_account(
    client: &Client,
    serum_program_id: &Pubkey,
    address: &Pubkey,
) -> Result<SerumMarketAccount> {
    let mut account = (*address, client.rpc().get_account(address).await?);
    let account_info = account.into_account_info();
    let market = Market::load(&account_info, serum_program_id, false)
        .with_context(|| format!("while deserializing serum market {address}"))?;

    Ok(SerumMarketAccount {
        base_mint: read_address(market.coin_mint),
        quote_mint: read_address(market.pc_mint),
        base_vault: read_address(market.coin_vault),
        quote_vault: read_address(market.pc_vault),
        request_queue: read_address(market.req_q),
        event_queue: read_address(market.event_q),
        bids: read_address(market.bids),
        asks: read_address(market.asks),
        quote_dust_threshold: market.pc_dust_threshold,
        base_lot_size: market.coin_lot_size,
        quote_lot_size: market.pc_lot_size,
        fee_rate_bps: market.fee_rate_bps,
    })
}

fn read_address(bytes: [u64; 4]) -> Pubkey {
    Pubkey::new(bytemuck::bytes_of(&bytes))
}
