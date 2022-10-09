use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use jet_margin_sdk::bonds::BondManager;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use solana_sdk::pubkey::Pubkey;

use crate::{
    client::Client,
    config::{
        BondMarketsDefinition, ConfigType, DependenciesDefinition, RpcDefinition,
        SerumMarketsDefinition, TokenDefinition,
    },
    serum::SerumMarketAccount,
};

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JetAppConfig {
    #[serde_as(as = "DisplayFromStr")]
    pub airspace_program_id: Pubkey,

    #[serde_as(as = "DisplayFromStr")]
    pub bonds_program_id: Pubkey,

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
    pub serum_markets: HashMap<String, SerumMarketInfo>,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirspaceInfo {
    pub name: String,
    pub tokens: Vec<String>,
    pub bond_markets: HashMap<String, BondMarketInfo>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faucet: Option<Pubkey>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub faucet_limit: Option<u64>,

    #[serde_as(as = "DisplayFromStr")]
    pub mint: Pubkey,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerumMarketInfo {
    pub symbol: String,
    pub base_symbol: String,
    pub quote_symbol: String,
    pub base_decimals: u8,
    pub quote_decimals: u8,

    #[serde_as(as = "DisplayFromStr")]
    pub market: Pubkey,

    #[serde(flatten)]
    pub market_info: SerumMarketAccount,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BondMarketInfo {
    pub symbol: String,

    #[serde_as(as = "DisplayFromStr")]
    pub bond_manager: Pubkey,

    #[serde(flatten)]
    pub market_info: BondManager,
}

impl JetAppConfig {
    pub async fn generate_from_config_dir(client: &Client, dir: &Path) -> Result<Self> {
        let rpc = Self::read_rpc_config(dir.join("rpc.toml")).await?;
        let deps = Self::read_dependency_programs(dir.join("dependencies.toml")).await?;
        let tokens = Self::generate_token_map_from_dir(client, dir.join("tokens")).await?;
        let serum_markets = Self::generate_market_map(
            client,
            &deps.serum_program_id,
            dir.join("serum-markets.toml"),
        )
        .await?;
        let bond_markets =
            Self::generate_bond_market_map(client, dir.join("bond-markets.toml")).await?;

        let airspaces = vec![AirspaceInfo {
            name: "default".to_owned(),
            tokens: tokens.keys().cloned().collect(),
            bond_markets,
        }];

        Ok(Self {
            airspace_program_id: jet_margin_sdk::jet_airspace::ID,
            bonds_program_id: jet_margin_sdk::jet_bonds::ID,
            control_program_id: jet_margin_sdk::jet_control::ID,
            margin_program_id: jet_margin_sdk::jet_margin::ID,
            margin_pool_program_id: jet_margin_sdk::jet_margin_pool::ID,
            margin_swap_program_id: jet_margin_sdk::jet_margin_swap::ID,
            metadata_program_id: jet_margin_sdk::jet_metadata::ID,
            margin_serum_program_id: Pubkey::default(),
            orca_swap_program_id: deps.orca_program_id,
            serum_program_id: deps.serum_program_id,
            faucet_program_id: deps.faucet_program_id,
            url: rpc.default,
            tokens,
            airspaces,
            serum_markets,
        })
    }

    async fn generate_token_map_from_dir(
        client: &Client,
        dir: PathBuf,
    ) -> Result<HashMap<String, TokenInfo>> {
        let mut token_infos = HashMap::new();
        let mut dir_contents = tokio::fs::read_dir(dir).await?;

        while let Some(entry) = dir_contents.next_entry().await? {
            if !entry.metadata().await?.is_file() {
                continue;
            }

            let token_def = Self::read_token_config(entry.path()).await?;
            let token_mint = client.read_mint(&token_def.config.mint).await?;

            token_infos.insert(
                token_def.token.symbol.clone(),
                TokenInfo {
                    symbol: token_def.token.symbol,
                    name: token_def.token.name,
                    mint: token_def.config.mint,
                    decimals: token_mint.decimals,
                    precision: token_def.token.precision,
                    faucet: token_def.token.faucet,
                    faucet_limit: token_def.token.faucet_limit,
                },
            );
        }

        Ok(token_infos)
    }

    async fn generate_market_map(
        client: &Client,
        serum_program_id: &Pubkey,
        path: PathBuf,
    ) -> Result<HashMap<String, SerumMarketInfo>> {
        let mut markets = HashMap::new();

        if !path.exists() {
            return Ok(markets);
        }

        let market_def = Self::read_serum_config(path).await?;

        for market in market_def.markets {
            let (base_symbol, quote_symbol) = parse_symbol_pair(&market.symbol_pair)?;
            let market_info =
                crate::serum::read_market_account(client, serum_program_id, &market.address)
                    .await?;
            let base_mint = client.read_mint(&market_info.base_mint).await?;
            let quote_mint = client.read_mint(&market_info.quote_mint).await?;

            markets.insert(
                format!("{base_symbol}_{quote_symbol}"),
                SerumMarketInfo {
                    symbol: market.symbol_pair,
                    base_decimals: base_mint.decimals,
                    quote_decimals: quote_mint.decimals,
                    base_symbol,
                    quote_symbol,
                    market: market.address,
                    market_info,
                },
            );
        }

        Ok(markets)
    }

    async fn generate_bond_market_map(
        client: &Client,
        path: PathBuf,
    ) -> Result<HashMap<String, BondMarketInfo>> {
        let mut bond_markets = HashMap::new();

        if !path.exists() {
            return Ok(bond_markets);
        }

        let market_def = Self::read_bonds_config(path).await?;

        for market in market_def.markets {
            let manager: BondManager = client.read_anchor_account(&market.manager).await?;
            bond_markets.insert(
                format!("{}_{}", market.symbol, market.tenor),
                BondMarketInfo {
                    symbol: market.symbol.clone(),
                    bond_manager: market.manager,
                    market_info: manager,
                },
            );
        }

        Ok(bond_markets)
    }

    async fn read_bonds_config(path: PathBuf) -> Result<BondMarketsDefinition> {
        match crate::config::read_config_file(&path)
            .await
            .with_context(|| format!("while reading bond market definition at {:?}", &path))?
        {
            ConfigType::BondMarkets(market_def) => Ok(market_def),
            _ => bail!("config {path:?} is not in the right format"),
        }
    }

    async fn read_token_config(path: PathBuf) -> Result<TokenDefinition> {
        match crate::config::read_config_file(&path)
            .await
            .with_context(|| format!("while reading token definition at {:?}", &path))?
        {
            ConfigType::Token(token_def) => Ok(token_def),
            _ => bail!("config {path:?} is not in the right format"),
        }
    }

    async fn read_serum_config(path: PathBuf) -> Result<SerumMarketsDefinition> {
        match crate::config::read_config_file(&path)
            .await
            .with_context(|| format!("while reading serum market definition at {:?}", &path))?
        {
            ConfigType::SerumMarkets(market_def) => Ok(market_def),
            _ => bail!("config {path:?} is not in the right format"),
        }
    }

    async fn read_rpc_config(path: PathBuf) -> Result<RpcDefinition> {
        match crate::config::read_config_file(&path)
            .await
            .with_context(|| format!("while reading rpc definition at {:?}", &path))?
        {
            ConfigType::Rpc(rpc_def) => Ok(rpc_def),
            _ => bail!("config {path:?} is not in the right format"),
        }
    }

    async fn read_dependency_programs(path: PathBuf) -> Result<DependenciesDefinition> {
        match crate::config::read_config_file(&path)
            .await
            .with_context(|| format!("while reading dependency definition at {:?}", &path))?
        {
            ConfigType::Dependencies(deps) => Ok(deps),
            _ => bail!("config {path:?} is not in the right format"),
        }
    }
}

fn parse_symbol_pair(pair: &str) -> Result<(String, String)> {
    let mut symbol_elements = pair.split('/');
    let (base, quote) = (symbol_elements.next(), symbol_elements.next());

    match (base, quote) {
        (Some(base), Some(quote)) => Ok((base.to_owned(), quote.to_owned())),
        _ => bail!("invalid serum-market symbol format: {pair}"),
    }
}
