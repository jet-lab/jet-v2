use anyhow::Result;
use jet_margin_sdk::{
    fixed_term::{event_queue_len, orderbook_slab_len, FixedTermIxBuilder, OrderbookAddresses},
    jet_fixed_term,
};
use jetctl::{
    actions::fixed_term::MarketParameters,
    client::{Client, ClientConfig, Plan},
    CliOpts, Command, FixedTermCommand,
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

    static ref PARAMS: MarketParameters = MarketParameters {
        borrow_tenor: 3,
        lend_tenor: 5,
        min_order_size: 1_000,
        seed: Pubkey::default().to_bytes().to_vec(),
        token_mint: USDC,
        token_oracle: Pubkey::default(),
        ticket_oracle: Pubkey::default(),
        event_queue: QUEUE_PATH.clone(),
        bids: BIDS_PATH.clone(),
        asks: ASKS_PATH.clone(),
        origination_fee: 10,
    };

    static ref OPTS: CliOpts = CliOpts {
        target_proposal: None,
        target_proposal_option: 0,
        airspace: None,
        compute_budget: None,
        dry_run: false,
        no_confirm: false,
        signer_path: Some(PAYER_PATH.clone()),
        rpc_endpoint: Some(ENDPOINT.to_string()),
        command: Command::Fixed { subcmd: FixedTermCommand::CreateMarket(PARAMS.clone()) },
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
    params: MarketParameters,
    queue_capacity: usize,
    book_capacity: usize,
) -> Result<(OrderbookAddresses, Plan)> {
    let eq = map_keypair_file(params.event_queue)?;
    let bids = map_keypair_file(params.bids)?;
    let asks = map_keypair_file(params.asks)?;

    let init_eq = {
        let rent = client
            .rpc()
            .get_minimum_balance_for_rent_exemption(event_queue_len(queue_capacity))
            .await?;
        solana_sdk::system_instruction::create_account(
            &client.signer()?,
            &eq.pubkey(),
            event_queue_len(queue_capacity) as u64,
            rent,
            &jet_fixed_term::ID,
        )
    };

    let rent = client
        .rpc()
        .get_minimum_balance_for_rent_exemption(orderbook_slab_len(book_capacity))
        .await?;
    let payer = client.signer()?;

    let init_bids = solana_sdk::system_instruction::create_account(
        &payer,
        &bids.pubkey(),
        rent,
        orderbook_slab_len(book_capacity) as u64,
        &jet_fixed_term::ID,
    );

    let init_asks = solana_sdk::system_instruction::create_account(
        &payer,
        &asks.pubkey(),
        rent,
        orderbook_slab_len(book_capacity) as u64,
        &jet_fixed_term::ID,
    );

    let steps = [
        format!("initialize-event-queue {}", eq.pubkey()),
        format!("initialize-bids-slab {}", bids.pubkey()),
        format!("initialize-asks-slab {}", asks.pubkey()),
    ];

    let orderbook = OrderbookAddresses {
        bids: bids.pubkey(),
        asks: asks.pubkey(),
        event_queue: eq.pubkey(),
    };

    Ok((
        orderbook,
        client
            .plan()?
            .instructions(
                [
                    Box::new(eq) as Box<dyn Signer>,
                    Box::new(bids),
                    Box::new(asks),
                ],
                steps,
                [init_eq, init_bids, init_asks],
            )
            .build(),
    ))
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
    let (orderbook, init_ob_accs) =
        create_orderbook_accounts(&client, PARAMS.clone(), QUEUE_CAPACITY, ORDERBOOK_CAPACITY)
            .await?;
    client.execute(init_ob_accs).await?;

    // init a usdc market
    let create_market =
        jetctl::actions::fixed_term::process_create_fixed_term_market(&client, PARAMS.clone())
            .await?;
    client.execute(create_market).await?;

    let fixed_term_market = FixedTermIxBuilder::new_from_seed(
        payer,
        &Pubkey::default(),
        &USDC,
        map_seed(PARAMS.seed.clone()),
        payer,
        PARAMS.token_oracle,
        PARAMS.ticket_oracle,
        None,
        orderbook,
    );

    // no-matching market
    let pause = client
        .plan()?
        .instructions(
            [],
            ["pause-market"],
            [fixed_term_market.pause_order_matching()],
        )
        .build();
    client.execute(pause).await?;

    Ok(())
}
