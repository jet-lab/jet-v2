use std::path::PathBuf;

use anchor_lang::prelude::Pubkey;
use anyhow::{Context, Result};
use clap::{AppSettings, Parser, Subcommand};
use client::{Client, ClientConfig};
use jet_margin_sdk::ix_builder::derive_airspace;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use solana_sdk::signer::Signer;

mod actions;
mod addresses;
mod client;

#[derive(Debug, Parser)]
#[clap(version)]
#[clap(propagate_version = true)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub struct CliOpts {
    // /// The relevant airspace to use
    // #[clap(global = true, long)]
    // pub airspace: Pubkey,
    /// Simulate transactions only
    #[clap(global = true, long)]
    pub dry_run: bool,

    /// Don't ask for confirmation
    #[clap(global = true, long)]
    pub no_confirm: bool,

    /// The path to the lookup registry authority keypair
    #[clap(global = true, long, short = 'a')]
    pub authority_path: Option<PathBuf>,

    /// The path to the signer to use (i.e. keypair or ledger-wallet)
    #[clap(global = true, long, short = 'k')]
    pub signer_path: Option<String>,

    /// The network endpoint to use
    #[clap(global = true, long, short = 'u')]
    pub rpc_endpoint: Option<String>,

    #[clap(subcommand)]
    pub command: Command,
}

#[serde_as]
#[derive(Debug, Subcommand, Deserialize)]
#[serde(tag = "action")]
pub enum Command {
    /// Create a new registry for an authority
    CreateRegistry,

    /// Update registry by adding any program accounts that do not exist.
    /// Useful when new pools or markets are added to an airspace.
    UpdateRegistry {
        #[clap(long)]
        airspace_name: String,
    },

    /// Close the registry, it should not have any lookup tables
    CloseRegistry,

    /// Remove a lookup table, disabling it first and then closing if it's disabled
    RemoveLookupTable {
        /// The lookup table address
        #[serde_as(as = "DisplayFromStr")]
        address: Pubkey,
    },
}

pub async fn run(opts: CliOpts) -> Result<()> {
    let _ = env_logger::builder().is_test(false).try_init();

    let rpc_endpoint = opts
        .rpc_endpoint
        .map(solana_clap_utils::input_validators::normalize_to_url_if_moniker);

    let authority_path = opts
        .authority_path
        .context("The authority keypair path must be set")?;

    let authority = solana_sdk::signature::read_keypair_file(&authority_path)
        .unwrap()
        .pubkey();

    let client_config = ClientConfig::new(
        opts.dry_run,
        opts.no_confirm,
        authority,
        opts.signer_path,
        rpc_endpoint,
    )?;
    let builder = client_config.get_builder();
    let client = Client::new(client_config).await?;

    let plan = match opts.command {
        Command::CreateRegistry => actions::create_registry(&client, &builder).await?,
        Command::UpdateRegistry { airspace_name } => {
            let airspace = derive_airspace(&airspace_name);
            actions::update_registry(&client, &builder, airspace).await?
        }
        Command::RemoveLookupTable { address } => {
            actions::remove_lookup_table(&client, &builder, address).await?
        }
        Command::CloseRegistry => actions::close_registry(&client, &builder).await?,
    };

    client.execute(plan).await?;

    Ok(())
}
