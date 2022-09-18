use std::{path::PathBuf, sync::Arc};

use actions::margin_pool::ConfigurePoolCliOptions;
use anchor_lang::prelude::Pubkey;
use anyhow::Result;
use clap::{AppSettings, Parser, Subcommand};
use client::{Client, ClientConfig};
use jet_rpc::{connection::RpcConnection, LOCALHOST_URL};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

mod actions;
mod anchor_ix_parser;
mod app_config;
mod client;
mod config;
mod governance;
mod serum;

#[derive(Debug, Parser)]
#[clap(version)]
#[clap(propagate_version = true)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub struct CliOpts {
    /// The target proposal to add transactions to, instead of executing them directly
    #[clap(global = true, long, value_parser, env = "JET_GOV_PROPOSAL_ID")]
    target_proposal: Option<Pubkey>,

    /// The target proposal option to add the transactions to
    #[clap(
        global = true,
        long,
        value_parser,
        env = "JET_GOV_PROPOSAL_OPT",
        default_value_t = 0
    )]
    target_proposal_option: u8,

    /// Prefix transactions with a change to the compute limit
    #[clap(global = true, long)]
    compute_budget: Option<u32>,

    /// Simulate transactions only
    #[clap(global = true, long)]
    dry_run: bool,

    /// The path to the signer to use (i.e. keypair or ledger-wallet)
    #[clap(global = true, long, short = 'k')]
    signer_path: Option<String>,

    /// The network endpoint to use
    #[clap(global = true, long, short = 'u')]
    rpc_endpoint: Option<String>,

    #[clap(subcommand)]
    command: Command,
}

#[serde_as]
#[derive(Debug, Subcommand, Deserialize)]
#[serde(tag = "action")]
enum Command {
    /// Deploy a program via governance/multisig
    ProgramDeploy {
        /// The address of the program to be upgraded
        #[clap(requires = "target-proposal")]
        program_id: Pubkey,

        /// The address of the buffer containing the new program code
        #[clap(long)]
        buffer: Pubkey,
    },

    /// Apply any changes in a config file, so that the network state reflects the config
    Apply {
        /// The path to the configuration to be applied
        config_path: PathBuf,
    },

    /// Generate the client app config file
    GenerateAppConfig {
        /// The path to the directory containing the config files
        config_dir: PathBuf,

        /// The path to write the generated file to
        #[clap(long, short = 'o')]
        output: PathBuf,
    },

    /// Inspect the transactions in a proposal
    InspectProposal {
        /// The address of the proposal to be inspected
        proposal_address: Pubkey,
    },

    /// Execute instructions on an approved proposal
    ExecuteProposal { proposal_address: Pubkey },

    /// Remove all instructions on a draft proposal
    ClearProposal {
        /// The draft proposal to remove instructions from
        proposal_address: Pubkey,
    },

    /// Create a new draft proposal
    CreateProposal {
        /// The name (or pubkey) of the governance to create a proposal for: 'eng', 'custody', 'dao'
        #[clap(long, short = 'g')]
        governance: String,

        /// The title/name for the proposal
        title: String,

        #[clap(long, default_value = "")]
        description: String,
    },

    /// Sign off on a draft proposal
    SignOffProposal {
        /// The address of the proposal to sign off
        proposal_address: Pubkey,
    },

    /// Approve an active proposal
    ApproveProposal {
        /// The address of the proposal to be approved
        proposal_address: Pubkey,
    },

    /// Ensure the authority account has been initialized
    CreateAuthority,

    /// Determine what type of metadata is set for an account (if any)
    CheckMetadata {
        /// The address that might have associated metadata
        address: Pubkey,
    },

    /// Register a new adapter for invocation through margin accounts
    RegisterAdapter {
        /// The program address to be used as an adapter
        address: Pubkey,
    },

    /// Add liquidator permissions
    AddLiquidator {
        /// The liquidator's address
        #[serde_as(as = "DisplayFromStr")]
        liquidator: Pubkey,
    },

    /// Remove liquidator permissions
    RemoveLiquidator {
        /// The liquidator's address
        #[serde_as(as = "DisplayFromStr")]
        liquidator: Pubkey,
    },

    /// Create a new margin pool for a token
    CreateMarginPool {
        /// The target token to create the pool for
        #[serde_as(as = "DisplayFromStr")]
        token: Pubkey,
    },

    /// Modify the parameters for an existing margin pool
    ConfigureMarginPool(ConfigurePoolCliOptions),

    /// Collect the fees for margin pools
    CollectMarginPoolFees,

    /// Show a summary of all margin pools
    ListMarginPools,

    /// List the top margin accounts by asset value
    ListTopMarginAccounts {
        /// The number of accounts to show
        #[clap(long, default_value_t = 10)]
        limit: usize,
    },
}

pub async fn run(opts: CliOpts) -> Result<()> {
    let rpc_endpoint = opts
        .rpc_endpoint
        .map(solana_clap_utils::input_validators::normalize_to_url_if_moniker)
        .unwrap_or(LOCALHOST_URL.into());
    let client_config =
        ClientConfig::new(opts.dry_run, false, opts.signer_path, opts.compute_budget)?;

    let rpc = RpcConnection::new(rpc_endpoint);
    let client = Client::new(Arc::new(rpc), client_config).await?;

    let mut plan = match opts.command {
        Command::ProgramDeploy { program_id, buffer } => {
            actions::program::process_deploy(
                &client,
                opts.target_proposal.unwrap(),
                program_id,
                buffer,
            )
            .await?
        }
        Command::Apply { config_path } => {
            actions::apply::process_apply(&client, config_path).await?
        }
        Command::GenerateAppConfig { config_dir, output } => {
            actions::global::process_generate_app_config(&client, &config_dir, &output).await?
        }
        Command::InspectProposal { proposal_address } => {
            actions::governance::process_proposal_inspect(&client, proposal_address).await?
        }
        Command::ClearProposal { proposal_address } => {
            actions::governance::process_proposal_clear_instructions(&client, proposal_address)
                .await?
        }
        Command::ExecuteProposal { proposal_address } => {
            actions::governance::process_proposal_execute(&client, proposal_address).await?
        }
        Command::CreateProposal {
            governance,
            title,
            description,
        } => {
            actions::governance::process_proposal_create(&client, &governance, title, description)
                .await?
        }
        Command::SignOffProposal { proposal_address } => {
            actions::governance::process_proposal_sign_off(&client, proposal_address).await?
        }
        Command::ApproveProposal { proposal_address } => {
            actions::governance::process_proposal_approve(&client, proposal_address).await?
        }
        Command::CreateAuthority => actions::global::process_create_authority(&client).await?,
        Command::CheckMetadata { address } => {
            actions::global::process_check_metadata(&client, address).await?
        }
        Command::RegisterAdapter { address } => {
            actions::margin::process_register_adapter(&client, address).await?
        }
        Command::AddLiquidator { liquidator } => {
            actions::margin::process_set_liquidator(&client, liquidator, true).await?
        }
        Command::RemoveLiquidator { liquidator } => {
            actions::margin::process_set_liquidator(&client, liquidator, false).await?
        }
        Command::CreateMarginPool { token } => {
            actions::margin_pool::process_create_pool(&client, token).await?
        }
        Command::ConfigureMarginPool(options) => {
            actions::margin_pool::process_configure_pool(&client, options).await?
        }
        Command::CollectMarginPoolFees => {
            actions::margin_pool::process_collect_pool_fees(&client).await?
        }
        Command::ListMarginPools => actions::margin_pool::process_list_pools(&client).await?,
        Command::ListTopMarginAccounts { limit } => {
            actions::margin::process_list_top_accounts(&client, limit).await?
        }
    };

    if let Some(proposal_id) = opts.target_proposal {
        println!(
            "targeting a proposal {proposal_id}, {} transactions will be added",
            plan.len()
        );

        plan = governance::convert_plan_to_proposal(
            &client,
            plan,
            proposal_id,
            opts.target_proposal_option,
        )
        .await?;
    }

    client.execute(plan).await?;

    Ok(())
}
