use clap::Parser;
use jetctl::CliOpts;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    if let Err(e) = jetctl::run(CliOpts::parse()).await {
        println!("error: ");

        for err in e.chain() {
            println!("{err}");
        }

        println!("{}", e.backtrace());
    }
    Ok(())
}
