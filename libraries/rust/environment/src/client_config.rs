use std::collections::HashSet;

use jet_instructions::{
    airspace::derive_airspace,
    fixed_term::derive_market_from_tenor,
    test_service::{derive_pyth_price, derive_spl_swap_pool, derive_token_mint},
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use solana_sdk::pubkey::Pubkey;

use crate::{
    builder::{resolve_token_mint, swap::resolve_swap_program, BuilderError},
    config::{AirspaceConfig, EnvironmentConfig, TokenDescription},
};

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

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl TryFrom<EnvironmentConfig> for JetAppConfig {
    type Error = BuilderError;

    fn try_from(env: EnvironmentConfig) -> Result<Self, Self::Error> {
        let mut seen = HashSet::new();
        let mut tokens = vec![];
        let mut airspaces = vec![];

        for airspace in &env.airspaces {
            for token in &airspace.tokens {
                if seen.contains(&token.name) {
                    continue;
                }

                seen.insert(token.name.clone());
                tokens.push(token.clone().into());
            }

            airspaces.push(airspace.clone().into());
        }

        let exchanges = env
            .exchanges
            .iter()
            .map(|dex| {
                let base = resolve_token_mint(&env, &dex.base)?;
                let quote = resolve_token_mint(&env, &dex.quote)?;
                let program = resolve_swap_program(env.network, &dex.program)?;

                let address = dex
                    .state
                    .unwrap_or_else(|| derive_spl_swap_pool(&program, &base, &quote).state);

                Ok(DexInfo {
                    program,
                    address,
                    base,
                    quote,
                })
            })
            .collect::<Result<_, BuilderError>>()?;

        Ok(Self {
            tokens,
            airspaces,
            exchanges,
        })
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

impl From<AirspaceConfig> for AirspaceInfo {
    fn from(config: AirspaceConfig) -> Self {
        let airspace = derive_airspace(&config.name);

        Self {
            name: config.name,
            tokens: config.tokens.iter().map(|t| t.name.clone()).collect(),
            fixed_term_markets: config
                .tokens
                .iter()
                .flat_map(|token| {
                    let mint = TokenInfo::from(token.clone()).mint;

                    token
                        .fixed_term_markets
                        .iter()
                        .map(move |m| derive_market_from_tenor(&airspace, &mint, m.borrow_tenor))
                })
                .collect(),
        }
    }
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

impl From<TokenDescription> for TokenInfo {
    fn from(desc: TokenDescription) -> Self {
        let mint = desc.mint.unwrap_or_else(|| derive_token_mint(&desc.name));
        let oracle = desc.pyth_price.unwrap_or_else(|| derive_pyth_price(&mint));

        Self {
            mint,
            oracle,
            symbol: desc.symbol,
            name: desc.name,
            decimals: desc.decimals,
            precision: desc.precision,
        }
    }
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

#[doc(hidden)]
pub mod legacy {
    use std::collections::HashMap;
    use thiserror::Error;

    use jet_instructions::fixed_term::Market;
    use jet_solana_client::{ExtError, NetworkUserInterface, NetworkUserInterfaceExt};

    use crate::programs::ORCA_V2;

    use super::*;

    #[derive(Error, Debug)]
    pub enum ConfigError<I: NetworkUserInterface> {
        #[error("ext error: {0}")]
        Ext(#[from] ExtError<I>),

        #[error("could not read market {0} on the network")]
        MissingMarket(Pubkey),
    }

    pub async fn from_config<I: NetworkUserInterface>(
        network: &I,
        config: &super::JetAppConfig,
    ) -> Result<JetAppConfig, ConfigError<I>> {
        let tokens = config
            .tokens
            .iter()
            .cloned()
            .map(|t| (t.name.clone(), TokenInfo::from(t)))
            .collect();

        let mut airspaces = vec![];

        for airspace in &config.airspaces {
            let mut fixed_term_markets = HashMap::new();

            for market_address in &airspace.fixed_term_markets {
                let Some(market_info) = network.get_anchor_account::<Market>(market_address).await? else {
                    return Err(ConfigError::MissingMarket(*market_address));
                };

                let token = config
                    .tokens
                    .iter()
                    .find(|t| t.mint == market_info.underlying_token_mint)
                    .unwrap_or_else(|| {
                        panic!(
                            "no matching token {} for market {market_address}",
                            market_info.underlying_token_mint
                        )
                    });
                let name = format!("{}_{}", token.name, market_info.borrow_tenor);

                fixed_term_markets.insert(
                    name,
                    FixedTermMarketInfo {
                        symbol: token.symbol.clone(),
                        market: *market_address,
                        market_info,
                    },
                );
            }

            airspaces.push(AirspaceInfo {
                name: airspace.name.clone(),
                tokens: airspace.tokens.clone(),
                fixed_term_markets,
            });
        }

        Ok(JetAppConfig {
            airspace_program_id: jet_instructions::airspace::AIRSPACE_PROGRAM,
            fixed_term_market_program_id: jet_instructions::fixed_term::FIXED_TERM_PROGRAM,
            control_program_id: jet_instructions::control::CONTROL_PROGRAM,
            margin_program_id: jet_instructions::margin::MARGIN_PROGRAM,
            margin_pool_program_id: jet_instructions::margin_pool::MARGIN_POOL_PROGRAM,
            margin_swap_program_id: jet_instructions::margin_swap::MARGIN_SWAP_PROGRAM,
            metadata_program_id: jet_metadata::ID,
            margin_serum_program_id: Pubkey::default(),
            orca_swap_program_id: ORCA_V2,
            serum_program_id: Pubkey::default(),
            faucet_program_id: None,
            url: String::new(),
            tokens,
            airspaces,
            swap_pools: vec![],
        })
    }

    #[serde_as]
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct JetAppConfig {
        #[serde_as(as = "DisplayFromStr")]
        pub airspace_program_id: Pubkey,

        #[serde_as(as = "DisplayFromStr")]
        pub fixed_term_market_program_id: Pubkey,

        #[serde_as(as = "DisplayFromStr")]
        pub control_program_id: Pubkey,

        #[serde_as(as = "DisplayFromStr")]
        pub margin_program_id: Pubkey,

        #[serde_as(as = "DisplayFromStr")]
        pub margin_pool_program_id: Pubkey,

        #[serde_as(as = "DisplayFromStr")]
        pub margin_serum_program_id: Pubkey,

        #[serde_as(as = "DisplayFromStr")]
        pub margin_swap_program_id: Pubkey,

        #[serde_as(as = "DisplayFromStr")]
        pub metadata_program_id: Pubkey,

        #[serde_as(as = "DisplayFromStr")]
        pub orca_swap_program_id: Pubkey,

        #[serde_as(as = "DisplayFromStr")]
        pub serum_program_id: Pubkey,

        #[serde_as(as = "Option<DisplayFromStr>")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub faucet_program_id: Option<Pubkey>,

        pub url: String,

        pub tokens: HashMap<String, TokenInfo>,
        pub airspaces: Vec<AirspaceInfo>,
        pub swap_pools: Vec<SwapPoolInfo>,
    }

    #[serde_as]
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AirspaceInfo {
        pub name: String,
        pub tokens: Vec<String>,
        pub fixed_term_markets: HashMap<String, FixedTermMarketInfo>,
    }

    #[serde_as]
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TokenInfo {
        pub symbol: String,
        pub name: String,
        pub decimals: u8,
        pub precision: u8,

        #[serde_as(as = "Option<DisplayFromStr>")]
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub faucet: Option<Pubkey>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub faucet_limit: Option<u64>,

        #[serde_as(as = "DisplayFromStr")]
        pub mint: Pubkey,
    }

    impl From<super::TokenInfo> for TokenInfo {
        fn from(other: super::TokenInfo) -> Self {
            Self {
                symbol: other.symbol,
                name: other.name,
                decimals: other.decimals,
                precision: other.precision,
                mint: other.mint,
                faucet: None,
                faucet_limit: None,
            }
        }
    }

    #[serde_as]
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SwapPoolInfo {
        #[serde_as(as = "DisplayFromStr")]
        pub swap_program: Pubkey,

        #[serde_as(as = "DisplayFromStr")]
        pub pool_state: Pubkey,

        #[serde_as(as = "DisplayFromStr")]
        pub token_a: Pubkey,

        #[serde_as(as = "DisplayFromStr")]
        pub token_b: Pubkey,
    }

    #[serde_as]
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct FixedTermMarketInfo {
        pub symbol: String,

        #[serde_as(as = "DisplayFromStr")]
        pub market: Pubkey,

        #[serde(flatten)]
        pub market_info: Market,
    }
}
