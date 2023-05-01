use anchor_lang::prelude::Pubkey;
use anyhow::Result;
use clap::{AppSettings, Parser, Subcommand};
use client::{Client, ClientConfig};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

mod actions;
mod addresses;
mod client;

#[derive(Debug, Parser)]
#[clap(version)]
#[clap(propagate_version = true)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub struct CliOpts {
    /// The relevant airspace to use
    #[clap(global = true, long)]
    pub airspace: Pubkey,

    /// The relevant authority to use
    #[clap(global = true, long)]
    pub authority: Pubkey,

    /// Simulate transactions only
    #[clap(global = true, long)]
    pub dry_run: bool,

    /// Don't ask for confirmation
    #[clap(global = true, long)]
    pub no_confirm: bool,

    /// The path to the signer to use (i.e. keypair or ledger-wallet)
    #[clap(global = true, long, short = 'a')]
    pub authority_path: Option<String>,

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
    UpdateRegistry,

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

    let client_config = ClientConfig::new(
        opts.dry_run,
        opts.no_confirm,
        opts.authority,
        opts.signer_path,
        rpc_endpoint,
    )?;
    let builder = client_config.get_builder();
    let client = Client::new(client_config).await?;

    let plan = match opts.command {
        Command::CreateRegistry => actions::create_registry(&client, &builder).await?,
        Command::UpdateRegistry => {
            actions::update_registry(&client, &builder, opts.airspace).await?
        }
        Command::RemoveLookupTable { address } => {
            actions::remove_lookup_table(&client, &builder, address).await?
        }
        Command::CloseRegistry => actions::close_registry(&client, &builder).await?,
    };

    client.execute(plan).await?;

    Ok(())
}
