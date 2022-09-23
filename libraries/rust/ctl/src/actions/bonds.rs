use anyhow::Result;
use clap::Parser;
use jet_margin_sdk::bonds::BondsIxBuilder;
use serde::{Deserialize, Serialize};
use solana_clap_utils::keypair::signer_from_path;
use solana_sdk::{pubkey::Pubkey, signer::Signer};

use crate::{
    client::{Client, Plan},
    governance::resolve_payer,
};

const MANAGER_VERSION: u64 = 0;

#[derive(Debug, Clone, Parser, Serialize, Deserialize)]
pub struct BondMarketParameters {
    #[clap(long)]
    pub duration: i64,

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

pub async fn process_create_bond_market<'a>(
    client: &Client,
    params: BondMarketParameters,
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
    let bonds = BondsIxBuilder::new_from_seed(&params.token_mint, seed, payer);

    let mut steps = vec![];
    let mut instructions = vec![];
    if client.account_exists(&bonds.manager()).await? {
        println!(
            "the manager for market [{}] already exists. Skipping initialization instruction",
            bonds.manager()
        );
    } else if !client.account_exists(&params.token_mint).await? {
        println!("the token {} does not exist", params.token_mint);
        return Ok(Plan::default());
    } else {
        let init_manager = bonds.initialize_manager(
            payer,
            MANAGER_VERSION,
            seed,
            params.duration,
            Pubkey::default(),
            Pubkey::default(),
        )?;
        steps.push(format!(
            "initialize-bond-manager for token [{}]",
            params.token_mint
        ));
        instructions.push(init_manager);
    }
    if client.account_exists(&bonds.orderbook_state()).await? {
        println!(
            "the market [{}] is already fully initialized",
            bonds.manager()
        );
        return Ok(Plan::default());
    }
    let init_orderbook = bonds.initialize_orderbook(
        payer,
        eq.pubkey(),
        bids.pubkey(),
        asks.pubkey(),
        params.min_order_size,
    )?;
    steps.push(format!(
        "initialize-order-book for bond market {}",
        bonds.manager()
    ));
    instructions.push(init_orderbook);

    let ob_accs = [eq, bids, asks];
    let signers: Vec<&dyn Signer> = ob_accs.iter().map(|b| (*b).as_ref()).collect();

    Ok(client
        .plan()?
        .instructions(signers, steps, instructions)
        .build())
}
