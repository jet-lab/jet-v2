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

use jet_bonds::orderbook::state::{event_queue_len, orderbook_slab_len};
use jet_control::TokenMetadataParams;
use jet_margin::TokenOracle;
use jet_test_service::TokenCreateParams;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey, rent::Rent, signature::Keypair, signer::Signer, system_instruction,
};

use crate::{
    bonds::BondsIxBuilder,
    ix_builder::{
        test_service::{self, derive_pyth_price, derive_pyth_product, derive_token_mint},
        MarginPoolConfiguration,
    },
    solana::transaction::TransactionBuilder,
    tx_builder::{global_initialize_instructions, AirspaceAdmin, TokenDepositsConfig},
};

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

    /// Bond markets (list of stake durations)
    #[serde(default)]
    pub bond_markets: Vec<BondMarketConfig>,
}

/// Configuration for bond markets
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct BondMarketConfig {
    /// The duration for staking
    pub duration: i64,

    /// The minimum order size for the AOB
    pub min_order_size: u64,
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
            utilization_rate_1: config.utilization_rate_1,
            utilization_rate_2: config.utilization_rate_2,
            borrow_rate_0: config.borrow_rate_0,
            borrow_rate_1: config.borrow_rate_1,
            borrow_rate_2: config.borrow_rate_2,
            borrow_rate_3: config.borrow_rate_3,
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

/// Describe an environment to initialize
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct EnvironmentConfig {
    /// The tokens to create
    pub tokens: Vec<TokenDescription>,

    /// The airspaces to create
    pub airspaces: Vec<AirspaceConfig>,

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

    txs.push(global_initialize_instructions(config.authority));

    txs.extend(create_token_tx(config));
    txs.extend(create_airspace_tx(config, rent)?);

    Ok(txs)
}

fn create_airspace_tx(
    config: &EnvironmentConfig,
    rent: &Rent,
) -> anyhow::Result<Vec<TransactionBuilder>> {
    let mut txs = vec![];

    for as_config in &config.airspaces {
        let as_admin = AirspaceAdmin::new(&as_config.name, config.authority, config.authority);

        txs.push(as_admin.create_airspace(as_config.is_restricted));

        let adapters = [jet_margin_pool::ID, jet_margin_swap::ID, jet_bonds::ID];

        txs.extend(
            adapters
                .into_iter()
                .map(|adapter| as_admin.configure_margin_adapter(adapter, true)),
        );

        for (name, tk_config) in &as_config.tokens {
            if !config.tokens.iter().any(|t| t.name == *name) {
                anyhow::bail!(
                    "configuring token {} in airspace, but not a global token",
                    name
                );
            }

            txs.extend(create_airspace_token_margin_config_tx(
                &as_admin, name, tk_config,
            ));
            txs.extend(create_airspace_token_bond_markets_tx(
                config, rent, &as_admin, name, tk_config,
            ));
        }
    }

    Ok(txs)
}

fn create_airspace_token_bond_markets_tx(
    config: &EnvironmentConfig,
    rent: &Rent,
    admin: &AirspaceAdmin,
    token_name: &str,
    tk_config: &AirspaceTokenConfig,
) -> Vec<TransactionBuilder> {
    let mut txs = vec![];

    for bm_config in &tk_config.bond_markets {
        let key_eq = Keypair::new();
        let key_bids = Keypair::new();
        let key_asks = Keypair::new();

        let len_eq = event_queue_len(EVENT_QUEUE_CAPACITY);
        let len_orders = orderbook_slab_len(ORDERBOOK_CAPACITY);

        let mut bond_manager_seed = [0u8; 32];
        bond_manager_seed[..8].copy_from_slice(&bm_config.duration.to_le_bytes());

        let mint = derive_token_mint(token_name);
        let bonds_ix = BondsIxBuilder::new_from_seed(
            &admin.airspace,
            &mint,
            bond_manager_seed,
            config.authority,
            derive_pyth_price(&mint),
            Pubkey::default(), //todo oracle
        );

        txs.push(TransactionBuilder {
            instructions: vec![
                system_instruction::create_account(
                    &config.authority,
                    &key_eq.pubkey(),
                    rent.minimum_balance(len_eq),
                    len_eq as u64,
                    &jet_bonds::ID,
                ),
                system_instruction::create_account(
                    &config.authority,
                    &key_bids.pubkey(),
                    rent.minimum_balance(len_orders),
                    len_orders as u64,
                    &jet_bonds::ID,
                ),
                system_instruction::create_account(
                    &config.authority,
                    &key_asks.pubkey(),
                    rent.minimum_balance(len_orders),
                    len_orders as u64,
                    &jet_bonds::ID,
                ),
                bonds_ix
                    .initialize_manager(config.authority, 0, bond_manager_seed, bm_config.duration)
                    .unwrap(),
                bonds_ix
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
