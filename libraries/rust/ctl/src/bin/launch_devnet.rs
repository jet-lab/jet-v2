use anyhow::Result;
use jetctl::{
    actions::bonds::BondMarketParameters,
    client::{Client, ClientConfig},
    CliOpts, Command,
};
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey, pubkey::Pubkey, signature::Keypair, signer::Signer,
};

lazy_static::lazy_static! {
    static ref PAYER_PATH: String = shellexpand::env("$PWD/tests/keypairs/payer.json")
    .unwrap().to_string();
    static ref PAYER: Keypair = map_keypair_file(shellexpand::env("$PWD/tests/keypairs/payer.json")
        .unwrap().to_string()).unwrap();
    static ref QUEUE: Keypair = map_keypair_file(shellexpand::env("$PWD/tests/keypairs/event_queue.json")
        .unwrap()
        .to_string()).unwrap();
    static ref BIDS: Keypair = map_keypair_file(shellexpand::env("$PWD/tests/keypairs/bids.json")
        .unwrap()
        .to_string()).unwrap();
    static ref ASKS: Keypair = map_keypair_file(shellexpand::env("$PWD/tests/keypairs/asks.json")
        .unwrap()
        .to_string()).unwrap();

        static ref PARAMS: BondMarketParameters = BondMarketParameters {
            duration: 5,
            min_order_size: 1_000,
            seed: Pubkey::default().to_bytes().to_vec(),
            token_mint: USDC,
            token_oracle: Pubkey::default(),
            ticket_oracle: Pubkey::default(),
            event_queue: QUEUE.pubkey(),
            bids: BIDS.pubkey(),
            asks: ASKS.pubkey(),
        };

        static ref OPTS: CliOpts = CliOpts {
            target_proposal: None,
            target_proposal_option: 0,
            compute_budget: None,
            dry_run: false,
            signer_path: Some(PAYER_PATH.clone()),
            rpc_endpoint: Some(ENDPOINT.to_string()),
            command: Command::CreateBondMarket(PARAMS.clone()),
        };
}
const USDC: Pubkey = pubkey!("4ruM7B4Hz4MUxy7DSFBRK9zCFLvkbLccB6S3zJ7t2525");
const ENDPOINT: &str = "https://api.devnet.solana.com";

fn map_keypair_file(path: String) -> Result<Keypair> {
    solana_clap_utils::keypair::keypair_from_path(&Default::default(), &path, "", false)
        .map_err(|_| anyhow::Error::msg("failed to read keypair"))
}

async fn airdrop_payer(client: &Client) -> Result<()> {
    loop {
        let sol = client.rpc().get_balance(&PAYER.pubkey()).await?;
        println!("Payer balance: {}", (sol as f64) / LAMPORTS_PER_SOL as f64);
        if sol >= 20 * LAMPORTS_PER_SOL {
            break;
        }
        if let Err(e) = client
            .rpc()
            .request_airdrop(&PAYER.pubkey(), 2 * LAMPORTS_PER_SOL)
            .await
        {
            println!("failed to obtain a full 20 sol airdrop.");
            println!("Final balance: {}", (sol as f64) / LAMPORTS_PER_SOL as f64);
            println!("Error: {e}");
            break;
        }
        println!("successful airdrop iteration...");
    }
    println!("Airdrop payer success!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client_config = ClientConfig::new(
        OPTS.dry_run,
        false,
        OPTS.signer_path.clone(),
        Some(ENDPOINT.to_string()),
        OPTS.compute_budget,
    )?;
    let client = Client::new(client_config).await?;
    airdrop_payer(&client).await?;
    Ok(())
}
