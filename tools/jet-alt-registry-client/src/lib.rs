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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    use jet_margin_sdk::ix_builder::derive_airspace;
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_sdk::{commitment_config::CommitmentConfig, signature::Keypair, signer::Signer};

    #[tokio::test]
    async fn test_create_registry() -> anyhow::Result<()> {
        let keypair = Keypair::new();
        let owner = keypair.pubkey();
        println!("Owner is {owner}");
        let rpc_url = "http://localhost:8899".to_string();
        let rpc = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::finalized());
        rpc.request_airdrop(&owner, 1_000_000_000).await?;

        // Save the keypair somewhere so we can read it back in
        let keypair_path = "/tmp/.kp_test_create_registry.json";
        let file = std::fs::File::create(keypair_path)?;
        serde_json::to_writer(file, &keypair.to_bytes().as_slice())?;

        let airspace = derive_airspace("default");

        let client_config = ClientConfig::new(
            false,
            true,
            owner,
            Some(keypair_path.to_string()),
            Some(rpc_url),
        )?;
        let builder = client_config.get_builder();
        let client = Client::new(client_config).await?;

        let plan = actions::create_registry(&client, &builder).await?;
        // The plan must have one step, being to create the registry
        assert_eq!(plan.entries.len(), 1);

        client.execute(plan).await?;

        tokio::time::sleep(Duration::from_secs(10)).await;

        let plan = actions::update_registry(&client, &builder, airspace).await?;
        // The plan must have a few steps
        assert!(plan.entries.len() >= 3);

        client.execute(plan).await?;

        Ok(())
    }
}
