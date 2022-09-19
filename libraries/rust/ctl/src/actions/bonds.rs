use anyhow::Result;
use clap::Parser;
use jet_bonds_lib::builder::BondsIxBuilder;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

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
    pub event_queue: Pubkey,

    #[clap(long)]
    pub bids: Pubkey,

    #[clap(long)]
    pub asks: Pubkey,
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

pub async fn process_create_bond_market(
    client: &Client,
    params: BondMarketParameters,
) -> Result<Plan> {
    let seed = map_seed(params.seed);
    let ix = BondsIxBuilder::new_from_seed(&params.token_mint, seed)
        .with_payer(&resolve_payer(client)?)
        .with_orderbook_accounts(
            Some(params.bids),
            Some(params.asks),
            Some(params.event_queue),
        );

    if client.account_exists(&ix.manager()).await? {
        println!("the pool already exists for token {}", params.token_mint);
        return Ok(Plan::default());
    }

    if !client.account_exists(&params.token_mint).await? {
        println!("the token {} does not exist", params.token_mint);
        return Ok(Plan::default());
    }
    let init_manager = ix.initialize_manager(
        MANAGER_VERSION,
        seed,
        params.duration,
        &params.token_mint,
        &params.token_oracle,
        &params.ticket_oracle,
    )?;
    let init_orderbook = ix.initialize_orderbook(params.min_order_size)?;
    Ok(client
        .plan()?
        .instructions(
            [],
            [
                format!("initialize-bond-manager for token {}", params.token_mint),
                format!("initialize-order-book for bond market {}", ix.manager()),
            ],
            [init_manager, init_orderbook],
        )
        .build())
}
