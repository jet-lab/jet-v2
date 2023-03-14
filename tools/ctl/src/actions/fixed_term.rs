use anyhow::Result;
use clap::Parser;
use jet_margin_sdk::fixed_term::{
    ix::recover_uninitialized, FixedTermIxBuilder, InitializeMarketParams, OrderbookAddresses,
    FIXED_TERM_PROGRAM,
};
use jet_program_common::{GOVERNOR_DEVNET, GOVERNOR_MAINNET};
use serde::{Deserialize, Serialize};
use solana_clap_utils::keypair::signer_from_path;
use solana_sdk::{pubkey::Pubkey, signer::Signer};

use crate::{
    client::{Client, NetworkKind, Plan},
    governance::resolve_payer,
};

const MANAGER_VERSION: u64 = 0;

#[derive(Debug, Clone, Parser, Serialize, Deserialize)]
pub struct MarketParameters {
    #[clap(long)]
    pub borrow_tenor: u64,

    #[clap(long)]
    pub lend_tenor: u64,

    #[clap(long)]
    pub origination_fee: u64,

    #[clap(long)]
    pub min_order_size: u64,

    #[clap(long)]
    pub seed: Vec<u8>,

    #[clap(long)]
    pub token_mint: Pubkey,

    #[clap(long)]
    pub token_oracle: Pubkey,

    #[clap(long)]
    pub ticket_oracle: Pubkey,

    #[clap(long)]
    pub event_queue: String,

    #[clap(long)]
    pub bids: String,

    #[clap(long)]
    pub asks: String,
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

pub async fn process_recover_uninitialized(client: &Client, recipient: Pubkey) -> Result<Plan> {
    let ft_accounts = client
        .rpc()
        .get_program_accounts(&FIXED_TERM_PROGRAM)
        .await?;
    let mut plan = client.plan()?;

    let governor = match client.network_kind {
        NetworkKind::Localnet => client.signer()?,
        NetworkKind::Devnet => GOVERNOR_DEVNET,
        NetworkKind::Mainnet => GOVERNOR_MAINNET,
    };

    for (address, account) in ft_accounts {
        if account.data[..8] != [0u8; 8] {
            continue;
        }

        plan = plan.instructions(
            [],
            [format!("recover {address}")],
            [recover_uninitialized(governor, address, recipient)],
        );
    }

    Ok(plan.build())
}

pub async fn process_create_fixed_term_market<'a>(
    client: &Client,
    params: MarketParameters,
) -> Result<Plan> {
    let payer = resolve_payer(client)?;
    let seed = map_seed(params.seed);
    let [eq, bids, asks] = [
        signer_from_path(
            &Default::default(),
            &params.event_queue,
            "event_queue",
            &mut None,
        )
        .map_err(|e| {
            anyhow::Error::msg(format!(
                "failed to resolve signer for event queue. Error: {e:?}"
            ))
        })?,
        signer_from_path(&Default::default(), &params.bids, "bids", &mut None).map_err(|e| {
            anyhow::Error::msg(format!("failed to resolve signer for bids. Error: {e:?}"))
        })?,
        signer_from_path(&Default::default(), &params.asks, "asks", &mut None).map_err(|e| {
            anyhow::Error::msg(format!("failed to resolve signer for asks. Error: {e:?}"))
        })?,
    ];
    let fixed_term_market = FixedTermIxBuilder::new_from_seed(
        client.signer()?,
        &Pubkey::default(),
        &params.token_mint,
        seed,
        payer,
        params.token_oracle,
        params.ticket_oracle,
        None,
        OrderbookAddresses {
            bids: bids.pubkey(),
            asks: asks.pubkey(),
            event_queue: eq.pubkey(),
        },
    );

    let mut steps = vec![];
    let mut instructions = vec![];
    if client.account_exists(&fixed_term_market.market()).await? {
        println!(
            "the fixed term market [{}] already exists. Skipping initialization instruction",
            fixed_term_market.market()
        );
    } else if !client.account_exists(&params.token_mint).await? {
        println!("the token {} does not exist", params.token_mint);
        return Ok(Plan::default());
    } else {
        if let Some(init_ata) = fixed_term_market.init_default_fee_destination(&payer) {
            instructions.push(init_ata);
        }
        let init_market = fixed_term_market.initialize_market(
            payer,
            InitializeMarketParams {
                version_tag: MANAGER_VERSION,
                seed,
                borrow_tenor: params.borrow_tenor,
                lend_tenor: params.lend_tenor,
                origination_fee: params.origination_fee,
            },
        );
        steps.push(format!(
            "initialize-market for token [{}]",
            params.token_mint
        ));
        instructions.push(init_market);
    }
    if client
        .account_exists(&fixed_term_market.orderbook_state())
        .await?
    {
        println!(
            "the market [{}] is already fully initialized",
            fixed_term_market.market()
        );
        return Ok(Plan::default());
    }
    let init_orderbook = fixed_term_market.initialize_orderbook(payer, params.min_order_size);
    steps.push(format!(
        "initialize-order-book for fixed term market {}",
        fixed_term_market.market()
    ));
    instructions.push(init_orderbook);

    let signers: Vec<Box<dyn Signer>> = vec![eq, bids, asks];

    Ok(client
        .plan()?
        .instructions(signers, steps, instructions)
        .build())
}
