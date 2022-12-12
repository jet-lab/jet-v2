mod client;
mod consumer;

use std::fs::read_to_string;

use anyhow::Result;
use clap::Parser;
use client::Client;
use consumer::Consumer;
use jet_margin_sdk::ix_builder::{derive_airspace, test_service::derive_token_mint};
use jetctl::actions::test::{derive_market_from_tenor_seed, TestEnvConfig};
use log::LevelFilter;
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config as LoggerConfig, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use solana_sdk::pubkey::Pubkey;

static LOCALNET_URL: &str = "http://127.0.0.1:8899";
static DEFAULT_LOG_PATH: &str = "crank-info.log";

#[derive(Parser, Debug)]
pub struct CliOpts {
    /// The filepath to the config file with market information
    #[clap(long, short = 'c')]
    pub config_path: String,

    /// The keypair to use for signing transactions
    #[clap(long, short = 'k')]
    pub keypair_path: Option<String>,

    /// The path to use for storing logs
    #[clap(long, short = 'l')]
    pub logfile_path: Option<String>,

    /// The rpc endpoint
    /// Defaults to localhost
    #[clap(long, short = 'u')]
    pub url: Option<String>,
}

async fn run(opts: CliOpts) -> Result<()> {
    let client = Client::new(
        opts.keypair_path,
        opts.url.unwrap_or_else(|| LOCALNET_URL.into()),
    )?;

    let targets = read_config(&opts.config_path)?;

    let mut consumers = vec![];
    for (_, markets) in targets {
        for market in markets {
            let c = client.clone();
            consumers.push(Consumer::spawn(c, market)?);
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
                    c.fixed_term_markets.into_iter().map(move |m| {
                        derive_market_from_tenor_seed(&airspace, &token_mint, m.borrow_tenor)
                    })
                })
                .collect::<Vec<_>>();

            (a.name, markets)
        })
        .collect::<Vec<_>>())
}

fn init_logger(log_path: Option<String>) -> Result<log4rs::Handle> {
    let stderr = ConsoleAppender::builder().target(Target::Stderr).build();
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(log_path.unwrap_or_else(|| DEFAULT_LOG_PATH.into()))?;

    let config = LoggerConfig::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Info)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Trace),
        )?;

    log4rs::init_config(config).map_err(anyhow::Error::from)
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = CliOpts::parse();

    // (Optional) TODO: add tools to be able to modify logging through this handle
    let _log_handle = init_logger(opts.logfile_path.clone())?;

    run(opts).await?;

    std::future::pending::<()>().await;
    Ok(())
}
