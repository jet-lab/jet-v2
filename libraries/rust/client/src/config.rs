use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use solana_sdk::pubkey::Pubkey;

#[serde_as]
#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JetAppConfig {
    pub tokens: Vec<TokenInfo>,
    pub airspaces: Vec<AirspaceInfo>,
    pub exchanges: Vec<DexInfo>,
}

impl JetAppConfig {
    pub const DEFAULT_URL: &'static str =
        "https://storage.googleapis.com/jet-app-config/config.json";
    pub const DEVNET_CONFIG_URL: &'static str = "https://storage.googleapis.com/jet-app-config/devnet.json";
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AirspaceInfo {
    pub name: String,
    pub tokens: Vec<String>,

    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub fixed_term_markets: Vec<Pubkey>,
}

impl Default for AirspaceInfo {
    fn default() -> Self {
        Self {
            name: "default".to_owned(),
            tokens: vec![],
            fixed_term_markets: vec![],
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub precision: u8,

    #[serde_as(as = "DisplayFromStr")]
    pub mint: Pubkey,

    #[serde_as(as = "DisplayFromStr")]
    pub oracle: Pubkey,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DexInfo {
    #[serde_as(as = "DisplayFromStr")]
    pub program: Pubkey,

    #[serde_as(as = "DisplayFromStr")]
    pub address: Pubkey,

    #[serde_as(as = "DisplayFromStr")]
    pub base: Pubkey,

    #[serde_as(as = "DisplayFromStr")]
    pub quote: Pubkey,
}
