use anyhow::Result;
use clap::Parser;
use jet_margin_sdk::bonds::event_builder::build_consume_events_info;
use jetctl::app_config::JetAppConfig;
use serde::Deserialize;

#[derive(Parser, Debug)]
pub struct CliOpts {
    /// The filepath to the config file with market information
    #[clap(long, short = 'c')]
    pub config_path: String,

    /// The keypair to use for signing transactions
    #[clap(long, short = 'k')]
    pub keypair_path: Option<String>,
}

#[tokio::main]
async fn main() {
    let opts = CliOpts::parse();
}

async fn run(opts: CliOpts) -> Result<()> {
    // load client
    // load market variables
    let config = JetAppConfig::deserialize()?;
    loop {
        let res = consume_events();
        // print res to log
    }
    Ok(())
}

fn consume_events() -> Result<()> {
    // load event queue
    let eq = todo!();
    let info = build_consume_events_info(eq)?;

    Ok(())
}
