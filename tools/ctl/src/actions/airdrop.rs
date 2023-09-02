use std::{collections::BTreeMap, path::PathBuf};

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use jet_instructions::rewards::airdrop;
use jet_rewards::{state::Airdrop, AirdropCreateParams};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use solana_sdk::{
    pubkey,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
};

use crate::client::{Client, Plan};

const JET_TOKEN: Pubkey = pubkey!("JET6zMJWkCN9tpRT2v2jfAmm5VnQFDpUBCyaKojmGtz");

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct AirdropCreateInfo {
    #[serde_as(as = "DisplayFromStr")]
    pub stake_pool: Pubkey,
    pub expire_at: DateTime<Utc>,
    pub short_desc: String,
    pub long_desc: String,

    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    pub recipients: BTreeMap<Pubkey, u64>,
}

#[serde_as]
#[derive(Debug, Subcommand, Deserialize)]
#[serde(tag = "airdrop")]
pub enum AirdropCommand {
    Create {
        #[clap(long)]
        incomplete: Option<Pubkey>,

        input: PathBuf,
    },

    Finalize {
        #[serde_as(as = "DisplayFromStr")]
        address: Pubkey,
    },
}

pub async fn run_command(client: &Client, command: AirdropCommand) -> Result<Plan> {
    match command {
        AirdropCommand::Create { incomplete, input } => {
            process_create(client, incomplete, input).await
        }

        AirdropCommand::Finalize { address } => process_finalize(client, address).await,
    }
}

async fn process_create(
    client: &Client,
    incomplete: Option<Pubkey>,
    input: PathBuf,
) -> Result<Plan> {
    let create_info: AirdropCreateInfo = serde_json::from_str(&std::fs::read_to_string(input)?)?;
    let create_params = AirdropCreateParams {
        expire_at: create_info.expire_at.timestamp(),
        stake_pool: create_info.stake_pool,
        short_desc: create_info.short_desc,
        long_desc: create_info.long_desc,
        flags: 0,
    };

    let mut plan = client.plan()?;
    let (airdrop_pubkey, max_index) = match incomplete {
        None => {
            let airdrop_state_key = Keypair::new();
            let airdrop_pubkey = airdrop_state_key.pubkey();
            let space = 8 + std::mem::size_of::<Airdrop>();

            plan = plan.instructions(
                [Box::new(airdrop_state_key) as Box<_>],
                [format!("create empty airdrop {}", airdrop_pubkey)],
                [
                    system_instruction::create_account(
                        &client.signer()?,
                        &airdrop_pubkey,
                        client
                            .rpc()
                            .get_minimum_balance_for_rent_exemption(space)
                            .await?,
                        space as u64,
                        &jet_rewards::ID,
                    ),
                    airdrop::create(
                        client.signer()?,
                        JET_TOKEN,
                        client.signer()?,
                        airdrop_pubkey,
                        create_params,
                    ),
                ],
            );

            (airdrop_pubkey, 0)
        }

        Some(existing) => {
            let existing_state = client.read_anchor_account::<Airdrop>(&existing).await?;
            let target = existing_state.target_info();

            (existing, target.recipients_total as usize)
        }
    };

    let recipients = create_info
        .recipients
        .into_iter()
        .skip(max_index)
        .collect::<Vec<_>>();

    let chunked_ix = airdrop::add_recipients(
        client.signer()?,
        airdrop_pubkey,
        recipients,
        max_index as u64,
    );

    for ix in chunked_ix {
        plan = plan.instructions([], ["add airdrop recipients"], [ix]);
    }

    Ok(plan.build())
}

async fn process_finalize(client: &Client, address: Pubkey) -> Result<Plan> {
    let airdrop = Box::new(client.read_anchor_account::<Airdrop>(&address).await?);
    let vault = client.read_token_account(&airdrop.reward_vault).await?;

    if airdrop.target_info().reward_total > vault.amount {
        anyhow::bail!("airdrop missing reward tokens, expected {} but found {}", airdrop.target_info().reward_total, vault.amount);
    }

    Ok(client
        .plan()?
        .instructions(
            [],
            ["finalize airdrop"],
            [airdrop::finalize(client.signer()?, address)],
        )
        .build())
}
