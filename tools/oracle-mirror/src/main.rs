use clap::Parser;
use jet_oracle_mirror::{run, CliOpts};

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
