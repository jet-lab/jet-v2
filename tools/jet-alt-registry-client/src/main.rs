use clap::Parser;
use jet_alt_registry_client::{run, CliOpts};

#[tokio::main]
async fn main() {
    let opts = CliOpts::parse();

    if let Err(e) = run(opts).await {
        println!("error: ");

        for err in e.chain() {
            println!("{err}");
        }

        println!("{}", e.backtrace());
    }
}
