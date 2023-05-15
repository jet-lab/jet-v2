use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use clap::Parser;

use solana_cli_config::{Config as SolanaConfig, CONFIG_FILE as SOLANA_CONFIG_FILE};
use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file};

use jet_margin_sdk::fixed_term::Crank;

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{prelude::*, EnvFilter};

use jet_environment::client_config::JetAppConfig;
use jet_solana_client::rpc::native::RpcConnection;

static LOCALNET_URL: &str = "http://127.0.0.1:8899";

#[derive(Parser, Debug)]
pub struct CliOpts {
    /// The filepath to the config file with market information
    #[clap(long, short = 'c')]
    pub config_path: PathBuf,

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

pub async fn run(opts: CliOpts) -> Result<()> {
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
    let rpc = Arc::new((
        RpcConnection::new(opts.url.as_deref().unwrap_or(LOCALNET_URL)),
        keypair,
    ));
    let targets = read_config(&opts.config_path)?;

    Crank::new(rpc, &targets).await?.run_forever().await;

    unreachable!("unexpected exit")
}

fn read_config(path: impl AsRef<Path>) -> Result<Vec<Pubkey>> {
    let app_json = std::fs::read_to_string(path)?;
    let app_config = serde_json::from_str::<JetAppConfig>(&app_json)?;

    Ok(app_config
        .airspaces
        .iter()
        .flat_map(|airspace| airspace.fixed_term_markets.iter().cloned())
        .collect())
}
