use std::{collections::HashMap, path::Path};

use anchor_lang::prelude::Rent;
use anyhow::{bail, Result};
use jet_margin_sdk::{
    bonds::bonds_pda,
    ix_builder::{
        derive_airspace,
        test_service::{derive_swap_pool, derive_token_mint},
    },
    test_service::{
        init_environment, AirspaceConfig, EnvironmentConfig, SwapPoolsConfig, TokenDescription,
    },
};
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey, pubkey::Pubkey, signer::Signer};

use crate::{
    app_config::{AirspaceInfo, BondMarketInfo, JetAppConfig, SwapPoolInfo, TokenInfo},
    client::{Client, NetworkKind, Plan},
};

pub async fn process_init_env(client: &Client, config_path: impl AsRef<Path>) -> Result<Plan> {
    if client.network_kind != NetworkKind::Localnet {
        bail!("running test init env on mainnet/devnet networks is not currently supported");
    }

    let env_config = read_env_config_from_file(client.signer()?, config_path)?;

    let txs = init_environment(&env_config, &Rent::default())?;
    let mut plan = client.plan()?;

    for tx in txs {
        plan = plan.instructions(
            tx.signers
                .into_iter()
                .map(|k| Box::new(k) as Box<dyn Signer>),
            [""],
            tx.instructions,
        );
    }

    Ok(plan.build())
}

pub async fn process_generate_app_config(
    client: &Client,
    config_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<Plan> {
    let env_config = read_env_config_from_file(client.signer()?, config_path)?;
    let app_config = generate_app_config(client, &env_config).await?;
    let app_json = serde_json::to_string_pretty(&app_config)?;

    std::fs::write(output_path, app_json)?;
    Ok(Plan::default())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TestEnvConfig {
    token: Vec<TokenDescription>,
    airspace: Vec<AirspaceConfig>,

    #[serde(default)]
    swap_pools: SwapPoolsConfig,
}

fn read_env_config_from_file(
    authority: Pubkey,
    path: impl AsRef<Path>,
) -> Result<EnvironmentConfig> {
    let config_content = std::fs::read_to_string(path)?;
    let config = toml::from_str::<TestEnvConfig>(&config_content)?;

    Ok(EnvironmentConfig {
        authority,
        tokens: config.token,
        airspaces: config.airspace,
        swap_pools: config.swap_pools,
    })
}

async fn generate_app_config(client: &Client, env: &EnvironmentConfig) -> Result<JetAppConfig> {
    let tokens = generate_token_app_config_from_env(env);
    let swap_pools = generate_swap_pool_app_config_from_env(env)?;
    let serum_markets = HashMap::new();

    let airspaces = futures::future::join_all(env.airspaces.iter().map(|as_config| async {
        AirspaceInfo {
            name: as_config.name.clone(),
            tokens: as_config.tokens.keys().cloned().collect(),
            bond_markets: generate_bond_markets_app_config_from_env(client, as_config)
                .await
                .unwrap(),
        }
    }))
    .await;

    Ok(JetAppConfig {
        airspace_program_id: jet_margin_sdk::jet_airspace::ID,
        bonds_program_id: jet_margin_sdk::jet_bonds::ID,
        control_program_id: jet_margin_sdk::jet_control::ID,
        margin_program_id: jet_margin_sdk::jet_margin::ID,
        margin_pool_program_id: jet_margin_sdk::jet_margin_pool::ID,
        margin_swap_program_id: jet_margin_sdk::jet_margin_swap::ID,
        metadata_program_id: jet_margin_sdk::jet_metadata::ID,
        margin_serum_program_id: Pubkey::default(),
        orca_swap_program_id: pubkey!("9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP"),
        serum_program_id: pubkey!("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin"),
        faucet_program_id: None,
        url: "http://127.0.0.1:8899".to_owned(),
        tokens,
        airspaces,
        swap_pools,
        serum_markets,
    })
}

fn generate_swap_pool_app_config_from_env(env: &EnvironmentConfig) -> Result<Vec<SwapPoolInfo>> {
    env.swap_pools
        .spl
        .iter()
        .map(|pair_string| {
            let sep_index = pair_string.find('/').ok_or_else(|| {
                anyhow::anyhow!(
                    "pool must be specified in 'A/B' format: (invalid: {}",
                    pair_string
                )
            })?;

            let (name_a, name_b) = pair_string.split_at(sep_index);
            let name_b = &name_b[1..];

            let token_a = derive_token_mint(name_a);
            let token_b = derive_token_mint(name_b);
            let pool = derive_swap_pool(&token_a, &token_b);

            Ok(SwapPoolInfo {
                swap_program: spl_token_swap::ID,
                pool_state: pool.state,
                token_a,
                token_b,
            })
        })
        .collect()
}

fn generate_token_app_config_from_env(env: &EnvironmentConfig) -> HashMap<String, TokenInfo> {
    HashMap::from_iter(env.tokens.iter().map(|desc| {
        (
            desc.name.clone(),
            TokenInfo {
                mint: derive_token_mint(&desc.name),
                symbol: desc.symbol.clone(),
                name: desc.name.clone(),
                decimals: desc.decimals,
                precision: desc.precision,
                faucet: None,
                faucet_limit: None,
            },
        )
    }))
}

async fn generate_bond_markets_app_config_from_env(
    client: &Client,
    as_config: &AirspaceConfig,
) -> Result<HashMap<String, BondMarketInfo>> {
    let mut bond_markets = HashMap::new();
    let airspace = derive_airspace(&as_config.name);

    for (name, config) in &as_config.tokens {
        let token_mint = derive_token_mint(name);

        for b_market in &config.bond_markets {
            let bond_manager = derive_bond_manager_from_duration_seed(
                &airspace,
                &token_mint,
                b_market.borrow_duration,
            );

            bond_markets.insert(
                format!("{name}_{}", b_market.borrow_duration),
                BondMarketInfo {
                    symbol: name.clone(),
                    bond_manager,
                    market_info: client.read_anchor_account(&bond_manager).await?,
                },
            );
        }
    }

    Ok(bond_markets)
}

fn derive_bond_manager(airspace: &Pubkey, token_mint: &Pubkey, seed: [u8; 32]) -> Pubkey {
    bonds_pda(&[
        jet_margin_sdk::jet_bonds::seeds::BOND_MANAGER,
        airspace.as_ref(),
        token_mint.as_ref(),
        &seed,
    ])
}

fn derive_bond_manager_from_duration_seed(
    airspace: &Pubkey,
    token_mint: &Pubkey,
    duration: i64,
) -> Pubkey {
    let mut seed = [0u8; 32];
    seed[..8].copy_from_slice(&duration.to_le_bytes());

    derive_bond_manager(airspace, token_mint, seed)
}
