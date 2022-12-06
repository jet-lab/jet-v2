use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use solana_sdk::pubkey::Pubkey;

use crate::actions::{margin::TokenConfig, margin_pool::MarginPoolParameters};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDefinition {
    pub token: TokenDescription,
    pub config: TokenConfig,
    pub margin_pool: MarginPoolParameters,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDescription {
    pub symbol: String,
    pub name: String,
    pub precision: u8,

    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub faucet: Option<Pubkey>,

    #[serde(default)]
    pub faucet_limit: Option<u64>,
}

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DependenciesDefinition {
    #[serde_as(as = "DisplayFromStr")]
    pub orca_program_id: Pubkey,

    #[serde_as(as = "DisplayFromStr")]
    pub serum_program_id: Pubkey,

    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub faucet_program_id: Option<Pubkey>,
}

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SerumMarketDefinition {
    #[serde_as(as = "DisplayFromStr")]
    pub address: Pubkey,

    pub symbol_pair: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SerumMarketsDefinition {
    #[serde(rename = "serum-market")]
    pub markets: Vec<SerumMarketDefinition>,
}

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FixedMarketDefinition {
    pub symbol: String,
    pub tenor: String,
    #[serde_as(as = "DisplayFromStr")]
    pub market: Pubkey,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FixedMarketsDefinition {
    #[serde(rename = "fixed-term-market")]
    pub markets: Vec<FixedMarketDefinition>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RpcDefinition {
    pub default: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigType {
    Token(TokenDefinition),
    Dependencies(DependenciesDefinition),
    SerumMarkets(SerumMarketsDefinition),
    FixedMarkets(FixedMarketsDefinition),
    Rpc(RpcDefinition),
}

pub async fn read_config_file(path: impl AsRef<Path>) -> Result<ConfigType> {
    let file_content = tokio::fs::read_to_string(path.as_ref())
        .await
        .with_context(|| format!("trying to read {:?}", path.as_ref()))?;

    Ok(toml::from_str(&file_content)?)
}
