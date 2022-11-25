mod client;
mod consumer;

use std::{fs::read_to_string, time::Duration};

use anyhow::Result;
use clap::Parser;
use client::{Client, AsyncSigner};
use consumer::Consumer;
use jet_margin_sdk::ix_builder::{derive_airspace, test_service::derive_token_mint};
use jetctl::actions::test::{derive_bond_manager_from_duration_seed, TestEnvConfig};
use solana_cli_config::{Config as SolanaConfig, CONFIG_FILE as SOLANA_CONFIG_FILE};
use solana_sdk::pubkey::Pubkey;


static LOCALNET_URL: &str = "http://127.0.0.1:8899";


#[derive(Parser, Debug)]
pub struct CliOpts {
    /// The filepath to the config file with market information
    #[clap(long, short = 'c')]
    pub config_path: String,

    /// The keypair to use for signing transactions
    #[clap(long, short = 'k')]
    pub keypair_path: Option<String>,

    /// The rpc endpoint
    /// Defaults to localhost
    #[clap(long, short = 'u')]
    pub url: Option<String>,
}

async fn run(opts: CliOpts) -> Result<()> {
    let client = Client::new(
        load_signer(opts.keypair_path)?,
        opts.url.unwrap_or_else(|| LOCALNET_URL.into()),
    );

    let targets = read_config(&opts.config_path)?;
    
    let mut consumers = vec![];
    for (_, markets) in targets {
        for market in markets {

            let c = client.clone();
            consumers.push(tokio::spawn(async move {
                Consumer::spawn(c, market).await
            }));
        }
    }
    
    Ok(())
}

fn read_config(path: &str) -> Result<Vec<(String, Vec<Pubkey>)>> {
    let cfg = read_to_string(path)?;
    Ok(toml::from_str::<TestEnvConfig>(&cfg)?
        .airspace
        .into_iter()
        .map(|a| {
            let markets = a
                .tokens
                .into_iter()
                .flat_map(|(t, c)| {
                    let airspace = derive_airspace(&a.name);
                    let token_mint = derive_token_mint(&t);
                    c.bond_markets.into_iter().map(move |m| {
                        derive_bond_manager_from_duration_seed(
                            &airspace,
                            &token_mint,
                            m.borrow_duration,
                        )
                    })
                })
                .collect::<Vec<_>>();

            (a.name, markets)
        })
        .collect::<Vec<_>>())
}

fn load_signer(path: Option<String>) -> Result<AsyncSigner> {
    let solana_config =
        SolanaConfig::load(SOLANA_CONFIG_FILE.as_ref().unwrap()).unwrap_or_default();
    solana_clap_utils::keypair::signer_from_path(
        &Default::default(),
        path.as_ref().unwrap_or(&solana_config.keypair_path),
        "wallet",
        &mut None,
    )
    .map(AsyncSigner::from)
    .map_err(|_| anyhow::Error::msg("failed to register signer from path"))
}

#[tokio::main]
async fn main() -> Result<()> {
    run(CliOpts::parse()).await?;

    std::future::pending::<()>().await;
    Ok(())
}
