use anyhow::Result;
use jet_bonds_sdk::builder::BondsIxBuilder;
use solana_sdk::pubkey::Pubkey;

use crate::client::{Client, Plan};

pub struct DeployBondMarketParameters {
    pub version_tag: u64,
    pub duration: i64,
    pub min_order_size: u64,
    pub seed: Vec<u8>,
    pub token_mint: Pubkey,
    pub program_authority: Pubkey,
    pub event_queue: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub payer: Option<Pubkey>,
    pub token_oracle: Option<Pubkey>,
    pub ticket_oracle: Option<Pubkey>,
}

fn map_seed(seed: Vec<u8>) -> [u8; 32] {
    let mut iter = seed.into_iter();
    let buf = &mut [0u8; 32];
    for i in 0..32 {
        match iter.next() {
            Some(b) => buf[i] = b,
            None => break,
        }
    }

    *buf
}

pub async fn process_deploy_manager(
    client: &Client,
    params: DeployBondMarketParameters,
) -> Result<Plan> {
    let seed = map_seed(params.seed);
    let ix = BondsIxBuilder::new_from_seed(&params.token_mint, seed)
        .with_payer(&client.signer()?)
        .with_authority(&params.program_authority);
    let init_manager = ix.initialize_manager(
        params.version_tag,
        seed,
        params.duration,
        &params.token_mint,
        &params.token_oracle.unwrap_or_default(),
        &params.ticket_oracle.unwrap_or_default(),
    )?;
    Ok(client
        .plan()?
        .instructions(
            [],
            [format!("initialize-manager {}", ix.manager())],
            [init_manager],
        )
        .build())
}
