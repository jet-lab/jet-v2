use std::{fs::read_to_string, path::PathBuf, sync::Arc, time::Duration};

use anyhow::Result;
use clap::Parser;

use jetctl::actions::test::{derive_market_from_tenor_seed, TestEnvConfig};
use solana_cli_config::{Config as SolanaConfig, CONFIG_FILE as SOLANA_CONFIG_FILE};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file, signer::Signer};

use jet_margin_sdk::{
    fixed_term::{
        event_consumer::{download_markets, EventConsumer},
        settler::settle_margin_users_loop,
        FixedTermIxBuilder,
    },
    ix_builder::{derive_airspace, test_service::derive_token_mint},
    util::no_dupe_queue::AsyncNoDupeQueue,
};
use jet_simulation::solana_rpc_api::RpcConnection;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{prelude::*, EnvFilter};

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

    /// Log file
    #[clap(long, short = 'l')]
    pub log_path: Option<PathBuf>,
}

async fn run(opts: CliOpts) -> Result<()> {
    let mut log_layers = vec![tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(EnvFilter::from_default_env())
        .boxed()];

    let _log_guard = opts.log_path.as_ref().map(|log_path| {
        std::fs::write(log_path, []).unwrap();

        let file_appender = RollingFileAppender::new(
            Rotation::NEVER,
            log_path.parent().unwrap(),
            log_path.file_name().unwrap(),
        );

        let (appender, guard) = tracing_appender::non_blocking(file_appender);
        log_layers.push(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_writer(appender)
                .with_filter(EnvFilter::from_default_env())
                .boxed(),
        );

        guard
    });

    tracing_subscriber::registry().with(log_layers).init();

    let solana_config =
        SolanaConfig::load(SOLANA_CONFIG_FILE.as_ref().unwrap()).unwrap_or_default();
    let keypair = read_keypair_file(
        opts.keypair_path
            .as_ref()
            .unwrap_or(&solana_config.keypair_path),
    )
    .unwrap();
    let payer = keypair.pubkey();
    let rpc = Arc::new(RpcConnection::new(
        keypair,
        RpcClient::new(LOCALNET_URL.to_string()),
    ));
    let targets = read_config(&opts.config_path)?;

    let markets = download_markets(rpc.as_ref(), &targets).await?;
    let consumer = EventConsumer::new(rpc.clone());
    for market in markets {
        let margin_accounts = AsyncNoDupeQueue::new();
        let ix = FixedTermIxBuilder::new_from_state(payer, &market);
        consumer.insert_market(market, Some(margin_accounts.clone()));
        tokio::spawn(settle_margin_users_loop(
            rpc.clone(),
            ix,
            margin_accounts,
            Default::default(),
        ));
    }

    loop {
        consumer.sync_users().await?;
        consumer.sync_queues().await?;

        while targets
            .iter()
            .any(|market| consumer.pending_events(market).unwrap() > 0)
        {
            consumer.consume().await?;
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

fn read_config(path: &str) -> Result<Vec<Pubkey>> {
    let cfg = read_to_string(path)?;
    Ok(toml::from_str::<TestEnvConfig>(&cfg)?
        .airspace
        .into_iter()
        .flat_map(|a| {
            a.tokens
                .iter()
                .flat_map(|(token_name, info)| {
                    info.fixed_term_markets.iter().map(|m| {
                        let airspace = derive_airspace(&a.name);
                        let token = derive_token_mint(token_name);

                        derive_market_from_tenor_seed(&airspace, &token, m.borrow_tenor)
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>())
}

#[tokio::main]
async fn main() -> Result<()> {
    run(CliOpts::parse()).await?;

    std::future::pending::<()>().await;
    Ok(())
}
