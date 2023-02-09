use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use thiserror::Error;

use solana_sdk::pubkey::Pubkey;

use jet_margin_pool::MarginPoolConfig;
use jet_solana_client::network::NetworkKind;

pub static DEFAULT_MARGIN_ADAPTERS: &[Pubkey] = &[
    jet_instructions::margin_swap::MARGIN_SWAP_PROGRAM,
    jet_instructions::margin_pool::MARGIN_POOL_PROGRAM,
    jet_instructions::fixed_term::FIXED_TERM_PROGRAM,
];

/// Description of errors that occur when reading configuration
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed while trying I/O on {path}: {error}")]
    IoError {
        path: PathBuf,
        error: std::io::Error,
    },

    #[error("failed while parsing toml in {path}: {error}")]
    Toml {
        path: PathBuf,
        error: toml::de::Error,
    },

    #[error("missing config directory for airspace {0}")]
    MissingAirspaceDir(PathBuf),
}
#[derive(Debug, Clone, PartialEq)]
pub struct EnvironmentConfig {
    /// The network this environment should exist within
    pub network: NetworkKind,

    /// List of programs that are allowed to be adapters in the margin system
    pub margin_adapters: Vec<Pubkey>,

    /// The authority allowed to adjust oracle prices in test environments
    pub oracle_authority: Option<Pubkey>,

    /// The airspaces that should exist for this environment
    pub airspaces: Vec<AirspaceConfig>,

    /// The DEX markets available to the environment
    pub exchanges: Vec<DexConfig>,
}

/// Describe an airspace to initialize
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AirspaceConfig {
    /// The name for the airspace
    pub name: String,

    /// If true, user registration with the airspace is restricted by an authority
    pub is_restricted: bool,

    /// The list of addresses authorized to act as cranks in the airspace
    pub cranks: Vec<Pubkey>,

    /// The tokens to be configured for use in the airspace
    pub tokens: Vec<TokenDescription>,
}

/// A description for a token to be created
#[serde_as]
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct TokenDescription {
    /// The symbol for the token
    pub symbol: String,

    /// The name for the token
    pub name: String,

    /// The number of decimals the token should have
    #[serde(default)]
    pub decimals: Option<u8>,

    /// The decimal precision when displaying token values
    pub precision: u8,

    /// The mint for the token (if it already exists)
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    pub mint: Option<Pubkey>,

    /// The pyth price acccount for the token (if it already exists)
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    pub pyth_price: Option<Pubkey>,

    /// The pyth product account for the token (if it already exists)
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    pub pyth_product: Option<Pubkey>,

    /// The maximum amount a user can request for an airdrop (when using test tokens)
    #[serde(default)]
    pub max_test_amount: Option<u64>,

    /// The adjustment of value for this token when used as collateral.
    pub collateral_weight: u16,

    /// The maximum leverage allowed for loans of this token.
    pub max_leverage: u16,

    /// The configuration to use for this token's margin pool (if it should exist)
    #[serde(default)]
    pub margin_pool: Option<MarginPoolConfig>,

    /// The configurations of fixed-term markets that should exist
    #[serde(default)]
    pub fixed_term_markets: Vec<FixedTermMarketConfig>,
}

/// Configuration for fixed term markets
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FixedTermMarketConfig {
    /// The tenor for borrows
    pub borrow_tenor: u64,

    /// The tenor for lending
    pub lend_tenor: u64,

    /// The origination fee for borrowing in origination_fee::FEE_UNIT
    pub origination_fee: u64,

    /// The minimum order size for the AOB
    pub min_order_size: u64,

    /// Whether or not order matching should be paused for the market
    #[serde(default)]
    pub paused: bool,

    /// The collateral adjustment value for tickets issued by this market
    #[serde(default)]
    pub ticket_collateral_weight: u16,

    /// The pyth price oracle for the ticket token
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    pub ticket_pyth_price: Option<Pubkey>,

    /// The pyth product oracle for the ticket token
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    pub ticket_pyth_product: Option<Pubkey>,

    /// The price of the tickets relative to the underlying
    /// multiplied by the underlying price to get the ticket price
    ///
    /// Only used in testing environments
    #[serde(default)]
    pub ticket_price: Option<f64>,
}

/// Information about a DEX available to an environment
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq)]
pub struct DexConfig {
    /// The program that does token exchanging
    pub program: String,

    /// A description for this DEX market/pool
    #[serde(default)]
    pub description: Option<String>,

    /// The address of the primary state account used for the token exchange
    #[serde(default)]
    pub state: Option<Pubkey>,

    /// The primary token in the pair that can be exchanged
    pub base: String,

    /// THe secondary token in the pair that can be exchanged
    pub quote: String,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
struct EnvRootAirspaceConfig {
    name: String,

    #[serde(default)]
    is_restricted: bool,

    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    cranks: Vec<Pubkey>,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
struct EnvRootConfigFile {
    network: NetworkKind,
    airspace: Vec<EnvRootAirspaceConfig>,

    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    margin_adapters: Vec<Pubkey>,

    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    oracle_authority: Option<Pubkey>,
}

pub fn read_env_config_dir(path: &Path) -> Result<EnvironmentConfig, ConfigError> {
    let root_file = path.join("env.toml");
    let dex_file = path.join("exchanges.toml");

    let root_content =
        std::fs::read_to_string(&root_file).map_err(|error| ConfigError::IoError {
            path: root_file.clone(),
            error,
        })?;
    let root =
        toml::from_str::<EnvRootConfigFile>(&root_content).map_err(|error| ConfigError::Toml {
            path: root_file.clone(),
            error,
        })?;

    let exchanges = read_dex_config_file(&dex_file)?;
    let airspaces = root
        .airspace
        .into_iter()
        .map(|config| {
            let airspace_config_path = path.join(&config.name);

            if !airspace_config_path.exists() || !airspace_config_path.is_dir() {
                return Err(ConfigError::MissingAirspaceDir(airspace_config_path));
            }

            read_airspace_dir(config, &airspace_config_path)
        })
        .collect::<Result<_, _>>()?;

    let margin_adapters = match root.margin_adapters.len() {
        0 => DEFAULT_MARGIN_ADAPTERS.to_vec(),
        _ => root.margin_adapters,
    };

    Ok(EnvironmentConfig {
        network: root.network,
        oracle_authority: root.oracle_authority,
        margin_adapters,
        airspaces,
        exchanges,
    })
}

fn read_airspace_dir(
    config: EnvRootAirspaceConfig,
    path: &Path,
) -> Result<AirspaceConfig, ConfigError> {
    let files = std::fs::read_dir(path)
        .map_err(|error| ConfigError::IoError {
            path: path.to_path_buf(),
            error,
        })?
        .filter_map(|entry| match entry {
            Err(error) => Some(Err(ConfigError::IoError {
                path: path.to_path_buf(),
                error,
            })),
            Ok(entry) if entry.path().extension().unwrap_or_default() == "toml" => {
                Some(Ok(entry.path()))
            }
            _ => None,
        })
        .collect::<Result<Vec<_>, _>>()?;

    let tokens = files
        .into_iter()
        .map(|f| read_token_desc_from_file(&f))
        .collect::<Result<_, _>>()?;

    Ok(AirspaceConfig {
        tokens,
        name: config.name,
        cranks: config.cranks,
        is_restricted: config.is_restricted,
    })
}

fn read_dex_config_file(path: &Path) -> Result<Vec<DexConfig>, ConfigError> {
    #[derive(Serialize, Deserialize)]
    struct DexConfigFile {
        dex: Vec<DexConfig>,
    }

    if !path.exists() {
        return Ok(vec![]);
    }

    let file_content = std::fs::read_to_string(path).map_err(|error| ConfigError::IoError {
        path: path.to_path_buf(),
        error,
    })?;
    let desc =
        toml::from_str::<DexConfigFile>(&file_content).map_err(|error| ConfigError::Toml {
            path: path.to_path_buf(),
            error,
        })?;

    Ok(desc.dex)
}

fn read_token_desc_from_file(path: &Path) -> Result<TokenDescription, ConfigError> {
    #[derive(Serialize, Deserialize)]
    struct FileTokenDesc {
        token: TokenDescription,
        #[serde(default)]
        margin_pool: Option<MarginPoolConfig>,
        #[serde(default)]
        fixed_term_market: Vec<FixedTermMarketConfig>,
    }

    let file_content = std::fs::read_to_string(path).map_err(|error| ConfigError::IoError {
        path: path.to_path_buf(),
        error,
    })?;
    let desc =
        toml::from_str::<FileTokenDesc>(&file_content).map_err(|error| ConfigError::Toml {
            path: path.to_path_buf(),
            error,
        })?;

    Ok(TokenDescription {
        margin_pool: desc.margin_pool,
        fixed_term_markets: desc.fixed_term_market,
        ..desc.token
    })
}
