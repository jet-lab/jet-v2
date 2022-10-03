use anyhow::Result;
use jet_margin_sdk::bonds::{event_queue_len, orderbook_slab_len, BondsIxBuilder};
use jetctl::{
    actions::bonds::BondMarketParameters,
    client::{Client, ClientConfig, Plan},
    BondsCommand, CliOpts, Command,
};
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey, pubkey::Pubkey, signature::Keypair, signer::Signer,
};

const USDC: Pubkey = pubkey!("4ruM7B4Hz4MUxy7DSFBRK9zCFLvkbLccB6S3zJ7t2525");
const ENDPOINT: &str = "https://api.devnet.solana.com";
const ORDERBOOK_CAPACITY: usize = 200;
const QUEUE_CAPACITY: usize = 400;

lazy_static::lazy_static! {
    static ref CONFIG_PATH: String = shellexpand::env("$PWD/target/config.json").unwrap().to_string();

    static ref PAYER_PATH: String = shellexpand::env("$PWD/tests/keypairs/payer.json")
    .unwrap().to_string();
    static ref QUEUE_PATH: String = shellexpand::env("$PWD/tests/keypairs/event_queue.json")
    .unwrap().to_string();
    static ref BIDS_PATH: String = shellexpand::env("$PWD/tests/keypairs/bids.json")
    .unwrap().to_string();
    static ref ASKS_PATH: String = shellexpand::env("$PWD/tests/keypairs/asks.json")
    .unwrap().to_string();

    static ref PARAMS: BondMarketParameters = BondMarketParameters {
        duration: 5,
        min_order_size: 1_000,
        seed: Pubkey::default().to_bytes().to_vec(),
        token_mint: USDC,
        token_oracle: Pubkey::default(),
        ticket_oracle: Pubkey::default(),
        event_queue: QUEUE_PATH.clone(),
        bids: BIDS_PATH.clone(),
        asks: ASKS_PATH.clone(),
    };

    static ref OPTS: CliOpts = CliOpts {
        target_proposal: None,
        target_proposal_option: 0,
        compute_budget: None,
        dry_run: false,
        signer_path: Some(PAYER_PATH.clone()),
        rpc_endpoint: Some(ENDPOINT.to_string()),
        command: Command::Bonds { subcmd: BondsCommand::CreateMarket(PARAMS.clone()) },
    };
}

fn map_keypair_file(path: String) -> Result<Keypair> {
    solana_clap_utils::keypair::keypair_from_path(&Default::default(), &path, "", false)
        .map_err(|_| anyhow::Error::msg("failed to read keypair"))
}

async fn airdrop_payer(client: &Client) -> Result<()> {
    let payer = client.signer()?;
    loop {
        let sol = client.rpc().get_balance(&payer).await?;
        println!("Payer balance: {}", (sol as f64) / LAMPORTS_PER_SOL as f64);
        if sol >= 20 * LAMPORTS_PER_SOL {
            break;
        }
        if let Err(e) = client
            .rpc()
            .request_airdrop(&payer, 2 * LAMPORTS_PER_SOL)
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

fn map_seed(seed: Vec<u8>) -> [u8; 32] {
    let mut buf = [0u8; 32];
    let mut iter = seed.into_iter();

    // clippy go away, I cant use `write` on a fixed array
    #[allow(clippy::needless_range_loop)]
    for i in 0..buf.len() {
        match iter.next() {
            Some(b) => buf[i] = b,
            None => break,
        }
    }

    buf
}

async fn create_orderbook_accounts(
    client: &Client,
    ix: &BondsIxBuilder,
    params: BondMarketParameters,
    queue_capacity: usize,
    book_capacity: usize,
) -> Result<Plan> {
    let eq = map_keypair_file(params.event_queue)?;
    let bids = map_keypair_file(params.bids)?;
    let asks = map_keypair_file(params.asks)?;

    let init_eq = {
        let rent = client
            .rpc()
            .get_minimum_balance_for_rent_exemption(event_queue_len(queue_capacity))
            .await?;
        ix.initialize_event_queue(&eq.pubkey(), queue_capacity, rent)?
    };

    let rent = client
        .rpc()
        .get_minimum_balance_for_rent_exemption(orderbook_slab_len(book_capacity))
        .await?;
    let init_bids = ix.initialize_orderbook_slab(&bids.pubkey(), book_capacity, rent)?;
    let init_asks = ix.initialize_orderbook_slab(&asks.pubkey(), book_capacity, rent)?;

    Ok(client
        .plan()?
        .instructions(
            [
                &eq as &dyn Signer,
                &bids as &dyn Signer,
                &asks as &dyn Signer,
            ],
            [
                format!("initialize-event-queue {}", eq.pubkey()),
                format!("initialize-bids-slab {}", bids.pubkey()),
                format!("initialize-asks-slab {}", asks.pubkey()),
            ],
            [init_eq, init_bids, init_asks],
        )
        .build())
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
    let payer = client.signer()?;

    // get us some sol
    airdrop_payer(&client).await?;

    // fund the ob accounts
    let bonds = BondsIxBuilder::new_from_seed(&USDC, map_seed(PARAMS.seed.clone()), payer)
        .with_payer(&payer);
    let init_ob_accs = create_orderbook_accounts(
        &client,
        &bonds,
        PARAMS.clone(),
        QUEUE_CAPACITY,
        ORDERBOOK_CAPACITY,
    )
    .await?;
    client.execute(init_ob_accs).await?;

    // init a usdc market
    let create_market =
        jetctl::actions::bonds::process_create_bond_market(&client, PARAMS.clone()).await?;
    client.execute(create_market).await?;

    // no-matching market
    let pause = client
        .plan()?
        .instructions([], ["pause-market"], [bonds.pause_order_matching()?])
        .build();
    client.execute(pause).await?;

    Ok(())
}
