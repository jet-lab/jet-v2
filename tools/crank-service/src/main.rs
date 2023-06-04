use anyhow::Result;

use clap::Parser;
use jet_fixed_term_crank::{run, CliOpts};

#[tokio::main]
async fn main() -> Result<()> {
    run(CliOpts::parse()).await?;

    std::future::pending::<()>().await;
    Ok(())
}
