// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright (C) 2022 JET PROTOCOL HOLDINGS, LLC.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::HashMap;

use jet_control::TokenMetadataParams;
use jet_margin::TokenOracle;
use jet_market::orderbook::state::{event_queue_len, orderbook_slab_len};
use jet_test_service::TokenCreateParams;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey, rent::Rent, signature::Keypair, signer::Signer, system_instruction,
};

use crate::{
    cat,
    fixed_term::FixedTermIxBuilder,
    ix_builder::{
        get_metadata_address,
        test_service::{
            self, derive_pyth_price, derive_pyth_product, derive_ticket_mint, derive_token_mint,
            if_not_initialized, spl_swap_pool_create,
        },
        ControlIxBuilder, MarginPoolConfiguration,
    },
    solana::transaction::TransactionBuilder,
    tx_builder::{global_initialize_instructions, AirspaceAdmin, TokenDepositsConfig},
};

static ADAPTERS: &[Pubkey] = &[jet_margin_pool::ID, jet_margin_swap::ID, jet_market::ID];
const ORDERBOOK_CAPACITY: usize = 1_000;
const EVENT_QUEUE_CAPACITY: usize = 1_000;

/// A description for a token to be created
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct TokenDescription {
    /// The symbol for the token
    pub symbol: String,

    /// The name for the token
    pub name: String,

    /// The number of decimals the token should have
    pub decimals: u8,

    /// The decimal precision when displaying token values
    pub precision: u8,
}

/// Token configuration within an airspace
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AirspaceTokenConfig {
    /// collateral weight
    pub collateral_weight: u16,

    /// max leverage
    pub max_leverage: u16,

    /// Margin pool config
    pub margin_pool_config: Option<MarginPoolConfig>,

    /// Fixed term markets (list of stake tenors)
    #[serde(default)]
    pub fixed_term_markets: Vec<FixedTermMarketConfig>,
}

/// Configuration for fixed term markets
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct FixedTermMarketConfig {
    /// The tenor for borrows
    pub borrow_tenor: i64,

    /// The tenor for lending
    pub lend_tenor: i64,

    /// The origination fee for borrowing in origination_fee::FEE_UNIT
    pub origination_fee: u64,

    /// The minimum order size for the AOB
    pub min_order_size: u64,

    /// Whether or not order matching should be paused for the market
    #[serde(default)]
    pub paused: bool,

    /// the price of the tickets relative to the underlying
    /// multiplied by the underlying price to get the ticket price
    pub ticket_price: String,
}

/// Configuration for margin pools
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct MarginPoolConfig {
    /// Space for binary settings
    pub flags: u64,

    /// The utilization rate at which first regime transitions to second
    pub utilization_rate_1: u16,

    /// The utilization rate at which second regime transitions to third
    pub utilization_rate_2: u16,

    /// The lowest borrow rate
    pub borrow_rate_0: u16,

    /// The borrow rate at the transition point from first to second regime
    pub borrow_rate_1: u16,

    /// The borrow rate at the transition point from second to third regime
    pub borrow_rate_2: u16,

    /// The highest possible borrow rate.
    pub borrow_rate_3: u16,

    /// The fee rate applied to interest payments collected
    pub management_fee_rate: u16,
}

impl From<MarginPoolConfig> for jet_margin_pool::MarginPoolConfig {
    fn from(config: MarginPoolConfig) -> Self {
        Self {
            flags: config.flags,
            utilization_rate_1: config.utilization_rate_1,
            utilization_rate_2: config.utilization_rate_2,
            borrow_rate_0: config.borrow_rate_0,
            borrow_rate_1: config.borrow_rate_1,
            borrow_rate_2: config.borrow_rate_2,
            borrow_rate_3: config.borrow_rate_3,
            management_fee_rate: config.management_fee_rate,
            ..Default::default()
        }
    }
}

/// Configuration for an airspace
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AirspaceConfig {
    /// The airspace name/seed
    pub name: String,

    /// Is the airspace restricted for users
    pub is_restricted: bool,

    /// The tokens to configure
    pub tokens: HashMap<String, AirspaceTokenConfig>,
}

/// Describe the swap pools to initialize
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq)]
pub struct SwapPoolsConfig {
    /// The token pairs to initialize SPL swap pools for
    #[serde(default)]
    pub spl: Vec<String>,
}

/// Describe an environment to initialize
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct EnvironmentConfig {
    /// The tokens to create
    pub tokens: Vec<TokenDescription>,

    /// The airspaces to create
    pub airspaces: Vec<AirspaceConfig>,

    /// The swap pools to create
    pub swap_pools: SwapPoolsConfig,

    /// The authority for all resources in the environment. Expected to sign all
    /// the intiailizer transactions, and pay for everything.
    pub authority: Pubkey,
}

/// Get transactions needed to initialize
pub fn init_environment(
    config: &EnvironmentConfig,
    rent: &Rent,
) -> anyhow::Result<Vec<TransactionBuilder>> {
    let mut txs = vec![];

    txs.extend(global_initialize_instructions(config.authority));
    txs.extend(create_global_adapter_register_tx(config.authority));
    txs.extend(create_token_tx(config));
    txs.extend(create_airspace_tx(config, rent)?);
    txs.extend(create_swap_pools_tx(config)?);

    Ok(txs)
}

/// Basic environment setup for hosted tests that has only the necessary global
/// state initialized
pub fn minimal_environment(authority: Pubkey) -> Vec<TransactionBuilder> {
    cat![
        global_initialize_instructions(authority),
        create_global_adapter_register_tx(authority),
    ]
}

fn create_global_adapter_register_tx(authority: Pubkey) -> Vec<TransactionBuilder> {
    let ctrl_ix = ControlIxBuilder::new(authority);
    ADAPTERS
        .iter()
        .map(|a| if_not_initialized(get_metadata_address(a), ctrl_ix.register_adapter(a)).into())
        .collect()
}

fn create_swap_pools_tx(config: &EnvironmentConfig) -> anyhow::Result<Vec<TransactionBuilder>> {
    let mut txs = vec![];

    for pair_string in &config.swap_pools.spl {
        let sep_index = pair_string.find('/').ok_or_else(|| {
            anyhow::anyhow!(
                "pool must be specified in 'A/B' format: (invalid: {}",
                pair_string
            )
        })?;

        let (name_a, name_b) = pair_string.split_at(sep_index);
        let name_b = &name_b[1..];

        verify_token_declared(config, name_a)?;
        verify_token_declared(config, name_b)?;

        let token_a = derive_token_mint(name_a);
        let token_b = derive_token_mint(name_b);

        txs.push(TransactionBuilder {
            instructions: vec![spl_swap_pool_create(
                &config.authority,
                &token_a,
                &token_b,
                8,
                500,
            )],
            signers: vec![],
        });
    }

    Ok(txs)
}

fn verify_token_declared(config: &EnvironmentConfig, name: &str) -> anyhow::Result<()> {
    if !config.tokens.iter().any(|t| t.name == *name) {
        anyhow::bail!(
            "configuring token {} in airspace, but not a global token",
            name
        );
    }

    Ok(())
}

fn create_airspace_tx(
    config: &EnvironmentConfig,
    rent: &Rent,
) -> anyhow::Result<Vec<TransactionBuilder>> {
    let mut txs = vec![];

    for as_config in &config.airspaces {
        let as_admin = AirspaceAdmin::new(&as_config.name, config.authority, config.authority);

        txs.push(as_admin.create_airspace(as_config.is_restricted));

        txs.extend(
            ADAPTERS
                .iter()
                .map(|adapter| as_admin.configure_margin_adapter(*adapter, true)),
        );

        for (name, tk_config) in &as_config.tokens {
            verify_token_declared(config, name)?;
            let token =
                config.tokens.iter().find(|t| &t.name == name).expect(
                    "cannot find a description for the token that needs a fixed term market",
                );

            txs.extend(create_airspace_token_margin_config_tx(
                &as_admin, name, tk_config,
            ));
            txs.extend(create_airspace_token_fixed_term_markets_tx(
                config, rent, &as_admin, token, tk_config,
            ));
        }
    }

    Ok(txs)
}

fn create_airspace_token_fixed_term_markets_tx(
    config: &EnvironmentConfig,
    rent: &Rent,
    admin: &AirspaceAdmin,
    token: &TokenDescription,
    tk_config: &AirspaceTokenConfig,
) -> Vec<TransactionBuilder> {
    let mut txs = vec![];

    for bm_config in &tk_config.fixed_term_markets {
        let key_eq = Keypair::new();
        let key_bids = Keypair::new();
        let key_asks = Keypair::new();

        let len_eq = event_queue_len(EVENT_QUEUE_CAPACITY);
        let len_orders = orderbook_slab_len(ORDERBOOK_CAPACITY);

        let mut market_seed = [0u8; 32];
        market_seed[..8].copy_from_slice(&bm_config.borrow_tenor.to_le_bytes());

        let mint = derive_token_mint(&token.name);
        let ticket_mint = derive_ticket_mint(&FixedTermIxBuilder::market_key(
            &admin.airspace,
            &mint,
            market_seed,
        ));
        let fixed_ix = FixedTermIxBuilder::new_from_seed(
            &admin.airspace,
            &mint,
            market_seed,
            config.authority,
            derive_pyth_price(&mint),
            derive_pyth_price(&ticket_mint),
            None,
        )
        .with_crank(&config.authority);

        txs.push(
            test_service::token_register(
                &config.authority,
                ticket_mint,
                &TokenCreateParams {
                    symbol: format!("{}_{}", token.symbol.clone(), bm_config.borrow_tenor),
                    name: format!("{}_{}", token.name.clone(), bm_config.borrow_tenor),
                    decimals: token.decimals,
                    authority: config.authority,
                    oracle_authority: config.authority,
                    max_amount: u64::MAX,
                    source_symbol: token.symbol.clone(),
                    price_ratio: bm_config.ticket_price.parse::<f64>().unwrap(),
                },
            )
            .into(),
        );

        txs.push(
            fixed_ix
                .init_default_fee_destination(&config.authority)
                .unwrap()
                .into(),
        );

        txs.push(TransactionBuilder {
            instructions: vec![
                system_instruction::create_account(
                    &config.authority,
                    &key_eq.pubkey(),
                    rent.minimum_balance(len_eq),
                    len_eq as u64,
                    &jet_market::ID,
                ),
                system_instruction::create_account(
                    &config.authority,
                    &key_bids.pubkey(),
                    rent.minimum_balance(len_orders),
                    len_orders as u64,
                    &jet_market::ID,
                ),
                system_instruction::create_account(
                    &config.authority,
                    &key_asks.pubkey(),
                    rent.minimum_balance(len_orders),
                    len_orders as u64,
                    &jet_market::ID,
                ),
                fixed_ix.initialize_market(
                    config.authority,
                    0,
                    market_seed,
                    bm_config.borrow_tenor,
                    bm_config.lend_tenor,
                    bm_config.origination_fee,
                ),
                fixed_ix
                    .initialize_orderbook(
                        config.authority,
                        key_eq.pubkey(),
                        key_bids.pubkey(),
                        key_asks.pubkey(),
                        bm_config.min_order_size,
                    )
                    .unwrap(),
            ],
            signers: vec![key_eq, key_bids, key_asks],
        });

        // Submit separately as it is large and causes tx to fail
        txs.push(TransactionBuilder {
            instructions: vec![fixed_ix.authorize_crank(config.authority).unwrap()],
            signers: vec![],
        });

        if bm_config.paused {
            txs.last_mut()
                .unwrap()
                .instructions
                .push(fixed_ix.pause_order_matching().unwrap());
        }

        txs.push(admin.register_fixed_term_market(
            mint,
            market_seed,
            tk_config.collateral_weight,
            tk_config.max_leverage,
        ));
    }

    txs
}

fn create_airspace_token_margin_config_tx(
    admin: &AirspaceAdmin,
    token_name: &str,
    config: &AirspaceTokenConfig,
) -> Vec<TransactionBuilder> {
    let mut txs = vec![];
    let mint = derive_token_mint(token_name);
    let pyth_price = derive_pyth_price(&mint);
    let pyth_product = derive_pyth_product(&mint);

    txs.push(admin.configure_margin_token_deposits(
        mint,
        Some(TokenDepositsConfig {
            collateral_weight: config.collateral_weight,
            oracle: TokenOracle::Pyth {
                price: pyth_price,
                product: pyth_product,
            },
        }),
    ));

    if let Some(margin_pool_config) = &config.margin_pool_config {
        txs.extend([
            admin.create_margin_pool(mint),
            admin.configure_margin_pool(
                mint,
                &MarginPoolConfiguration {
                    metadata: Some(TokenMetadataParams {
                        token_kind: jet_metadata::TokenKind::Collateral,
                        collateral_weight: config.collateral_weight,
                        max_leverage: config.max_leverage,
                    }),
                    parameters: Some(margin_pool_config.clone().into()),
                    pyth_product: Some(pyth_product),
                    pyth_price: Some(pyth_price),
                },
            ),
        ]);
    }

    txs
}

fn create_token_tx(config: &EnvironmentConfig) -> Vec<TransactionBuilder> {
    config
        .tokens
        .iter()
        .map(|desc| {
            let ix = match &*desc.name {
                "SOL" => test_service::token_init_native(&config.authority, &config.authority),
                _ => test_service::token_create(
                    &config.authority,
                    &TokenCreateParams {
                        symbol: desc.symbol.clone(),
                        name: desc.name.clone(),
                        decimals: desc.decimals,
                        authority: config.authority,
                        oracle_authority: config.authority,
                        max_amount: u64::MAX,
                        source_symbol: desc.symbol.clone(),
                        price_ratio: 1.0,
                    },
                ),
            };

            TransactionBuilder {
                instructions: vec![ix],
                signers: vec![],
            }
        })
        .collect()
}
