
use std::{path::PathBuf, collections::HashMap};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use clap::Subcommand;
use serde_with::{serde_as, DisplayFromStr};

use solana_sdk::pubkey::Pubkey;

use crate::client::{Client, Plan};

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct AirdropCreateInfo {
    #[serde_as(as = "DisplayFromStr")]
    pub stake_pool: Pubkey,
    pub expire_at: DateTime<Utc>,
    pub short_desc: String,
    pub long_desc: String,

    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub recipients: HashMap<Pubkey, u64>,
}


#[serde_as]
#[derive(Debug, Subcommand, Deserialize)]
#[serde(tag = "fixed-term-market-action")]
pub enum AirdropCommand {
    Create {
        input: PathBuf
    }
}

async fn run_command(client: &Client, command: AirdropCommand) -> Result<Plan> {
    match command {
        AirdropCommand::Create { input } => process_create(client, input).await
    }
}

async fn process_create(client: &Client, input: PathBuf) -> Result<Plan> {
    Ok(Plan::default())
}
