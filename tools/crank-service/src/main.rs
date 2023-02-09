use anyhow::Result;

use clap::Parser;
use jet_fixed_terms_crank_service::{run, CliOpts};

#[tokio::main]
async fn main() -> Result<()> {
    run(CliOpts::parse()).await?;

    std::future::pending::<()>().await;
    Ok(())
}
