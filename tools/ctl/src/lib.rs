use std::path::PathBuf;

use actions::{bonds::BondMarketParameters, margin_pool::ConfigurePoolCliOptions};
use anchor_lang::prelude::Pubkey;
use anyhow::Result;
use clap::{AppSettings, Parser, Subcommand};
use client::{Client, ClientConfig, Plan};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

pub mod actions;
pub mod app_config;
pub mod client;
pub mod config;

mod anchor_ix_parser;
mod governance;
mod serum;

#[derive(Debug, Parser)]
#[clap(version)]
#[clap(propagate_version = true)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub struct CliOpts {
    /// The target proposal to add transactions to, instead of executing them directly
    #[clap(global = true, long, value_parser, env = "JET_GOV_PROPOSAL_ID")]
    pub target_proposal: Option<Pubkey>,

    /// The target proposal option to add the transactions to
    #[clap(
        global = true,
        long,
        value_parser,
        env = "JET_GOV_PROPOSAL_OPT",
        default_value_t = 0
    )]
    pub target_proposal_option: u8,

    /// Prefix transactions with a change to the compute limit
    #[clap(global = true, long)]
    pub compute_budget: Option<u32>,

    /// Simulate transactions only
    #[clap(global = true, long)]
    pub dry_run: bool,

    /// Don't ask for confirmation
    #[clap(global = true, long)]
    pub no_confirm: bool,

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
#[serde(tag = "proposal-action")]
pub enum ProposalsCommand {
    /// Inspect the transactions in a proposal
    Inspect {
        /// The address of the proposal to be inspected
        proposal_address: Pubkey,
    },

    /// Execute instructions on an approved proposal
    Execute { proposal_address: Pubkey },

    /// Remove all instructions on a draft proposal
    Clear {
        /// The draft proposal to remove instructions from
        proposal_address: Pubkey,
    },

    /// Create a new draft proposal
    Create {
        /// The name (or pubkey) of the governance to create a proposal for: 'eng', 'custody', 'dao'
        #[clap(long, short = 'g')]
        governance: String,

        /// The title/name for the proposal
        title: String,

        #[clap(long, default_value = "")]
        description: String,
    },

    /// Sign off on a draft proposal, allowing it to be voted on
    SignOff {
        /// The address of the proposal to sign off
        proposal_address: Pubkey,
    },

    /// Finalize a completed proposal
    Finalize {
        /// The address of the proposal to be finalized
        proposal_address: Pubkey,
    },

    /// Approve an active proposal
    Approve {
        /// The address of the proposal to be approved
        proposal_address: Pubkey,
    },
}

#[serde_as]
#[derive(Debug, Subcommand, Deserialize)]
#[serde(tag = "margin-action")]
pub enum MarginCommand {
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

    /// Update the metadata for existing positions
    RefreshPositionMd {
        /// The token that had its config updated
        token: Pubkey,
    },

    /// Update all the balances for positions on an account
    UpdateBalances {
        /// The account to have its balances updated
        account: Pubkey,
    },

    /// Transfer a position owned directly by an account
    TransferPosition {
        /// The source margin account to transfer out of
        #[clap(long, short = 's')]
        #[serde_as(as = "DisplayFromStr")]
        source: Pubkey,

        /// The target margin account to transfer into
        #[clap(long, short = 't')]
        #[serde_as(as = "DisplayFromStr")]
        target: Pubkey,

        /// The target token to be transferred
        #[clap(long)]
        #[serde_as(as = "DisplayFromStr")]
        token: Pubkey,

        /// The amount to transfer. Default is to transfer the entire position
        amount: Option<u64>,
    },

    /// List the top margin accounts by asset value
    ListTopAccounts {
        /// The number of accounts to show
        #[clap(long, default_value_t = 10)]
        limit: usize,
    },

    /// Display a detailed view of each margin account
    Inspect {
        /// List of accounts to inspect
        addresses: Vec<Pubkey>,
    },
}

#[serde_as]
#[derive(Debug, Subcommand, Deserialize)]
#[serde(tag = "margin-pool-action")]
pub enum MarginPoolCommand {
    /// Create a new margin pool for a token
    Create {
        /// The target token to create the pool for
        #[serde_as(as = "DisplayFromStr")]
        token: Pubkey,
    },

    /// Modify the parameters for an existing margin pool
    Configure(ConfigurePoolCliOptions),

    /// Collect the fees for margin pools
    CollectFees,

    /// Transfer a loan between margin accounts
    TransferLoan {
        /// The source margin account to transfer out of
        #[clap(long, short = 's')]
        #[serde_as(as = "DisplayFromStr")]
        source: Pubkey,

        /// The target margin account to transfer into
        #[clap(long, short = 't')]
        #[serde_as(as = "DisplayFromStr")]
        target: Pubkey,

        /// The target token to be transferred
        #[clap(long)]
        #[serde_as(as = "DisplayFromStr")]
        token: Pubkey,

        /// The amount to transfer. Default is to transfer the entire position
        amount: Option<u64>,
    },

    /// Show a summary of all margin pools
    List,

    /// Show configuration for a pool
    Show {
        /// The token to show the pool for
        token: Pubkey,
    },
}

#[serde_as]
#[derive(Debug, Subcommand, Deserialize)]
#[serde(tag = "bonds-action")]
pub enum BondsCommand {
    /// Create a new bond market
    CreateMarket(BondMarketParameters),
}

#[serde_as]
#[derive(Debug, Subcommand, Deserialize)]
#[serde(tag = "test-action")]
pub enum TestCommand {
    /// Initialize a test environment
    InitEnv {
        /// A config file specifying the resources to be intiailized
        config_path: PathBuf,
    },

    /// Generate the application config from a test environment
    GenerateAppConfig {
        /// The config path used to initialize the environment
        config_path: PathBuf,

        /// The output file path for the generated config
        #[clap(long, short = 'o')]
        output: PathBuf,
    },
}

#[serde_as]
#[derive(Debug, Subcommand, Deserialize)]
#[serde(tag = "action")]
pub enum Command {
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

    /// Proposal management
    Proposals {
        #[clap(subcommand)]
        subcmd: ProposalsCommand,
    },

    /// Ensure the authority account has been initialized
    CreateAuthority,

    /// Determine what type of metadata is set for an account (if any)
    CheckMetadata {
        /// The address that might have associated metadata
        address: Pubkey,
    },

    /// Margin account management
    Margin {
        #[clap(subcommand)]
        subcmd: MarginCommand,
    },

    /// Margin pool management
    MarginPool {
        #[clap(subcommand)]
        subcmd: MarginPoolCommand,
    },

    /// Bond market management
    Bonds {
        #[clap(subcommand)]
        subcmd: BondsCommand,
    },

    /// Test management
    Test {
        #[clap(subcommand)]
        subcmd: TestCommand,
    },
}

pub async fn run(opts: CliOpts) -> Result<()> {
    let rpc_endpoint = opts
        .rpc_endpoint
        .map(solana_clap_utils::input_validators::normalize_to_url_if_moniker);
    let client_config = ClientConfig::new(
        opts.dry_run,
        opts.no_confirm,
        opts.signer_path,
        rpc_endpoint,
        opts.compute_budget,
    )?;
    let client = Client::new(client_config).await?;

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
        Command::Proposals { subcmd } => run_proposals_command(&client, subcmd).await?,
        Command::CreateAuthority => actions::global::process_create_authority(&client).await?,
        Command::CheckMetadata { address } => {
            actions::global::process_check_metadata(&client, address).await?
        }
        Command::Margin { subcmd } => run_margin_command(&client, subcmd).await?,
        Command::MarginPool { subcmd } => run_margin_pool_command(&client, subcmd).await?,
        Command::Bonds { subcmd } => run_bonds_command(&client, subcmd).await?,
        Command::Test { subcmd } => run_test_command(&client, subcmd).await?,
    };

    if let Some(proposal_id) = opts.target_proposal {
        println!(
            "targeting a proposal {proposal_id}, {} transactions will be added",
            plan.entries.len()
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

async fn run_proposals_command(client: &Client, command: ProposalsCommand) -> Result<Plan> {
    match command {
        ProposalsCommand::Inspect { proposal_address } => {
            actions::governance::process_proposal_inspect(client, proposal_address).await
        }
        ProposalsCommand::Clear { proposal_address } => {
            actions::governance::process_proposal_clear_instructions(client, proposal_address).await
        }
        ProposalsCommand::Execute { proposal_address } => {
            actions::governance::process_proposal_execute(client, proposal_address).await
        }
        ProposalsCommand::Create {
            governance,
            title,
            description,
        } => {
            actions::governance::process_proposal_create(client, &governance, title, description)
                .await
        }
        ProposalsCommand::SignOff { proposal_address } => {
            actions::governance::process_proposal_sign_off(client, proposal_address).await
        }
        ProposalsCommand::Finalize { proposal_address } => {
            actions::governance::process_proposal_finalize(client, proposal_address).await
        }
        ProposalsCommand::Approve { proposal_address } => {
            actions::governance::process_proposal_approve(client, proposal_address).await
        }
    }
}

async fn run_margin_command(client: &Client, command: MarginCommand) -> Result<Plan> {
    match command {
        MarginCommand::RegisterAdapter { address } => {
            actions::margin::process_register_adapter(client, address).await
        }
        MarginCommand::AddLiquidator { liquidator } => {
            actions::margin::process_set_liquidator(client, liquidator, true).await
        }
        MarginCommand::RemoveLiquidator { liquidator } => {
            actions::margin::process_set_liquidator(client, liquidator, false).await
        }
        MarginCommand::RefreshPositionMd { token } => {
            actions::margin::process_refresh_metadata(client, token).await
        }
        MarginCommand::UpdateBalances { account } => {
            actions::margin::process_update_balances(client, account).await
        }
        MarginCommand::TransferPosition {
            source,
            target,
            token,
            amount,
        } => {
            actions::margin::process_transfer_position(client, source, target, token, amount).await
        }
        MarginCommand::ListTopAccounts { limit } => {
            actions::margin::process_list_top_accounts(client, limit).await
        }
        MarginCommand::Inspect { addresses } => {
            actions::margin::process_inspect(client, addresses).await
        }
    }
}

async fn run_margin_pool_command(client: &Client, command: MarginPoolCommand) -> Result<Plan> {
    match command {
        MarginPoolCommand::Create { token } => {
            actions::margin_pool::process_create_pool(client, token).await
        }
        MarginPoolCommand::Configure(options) => {
            actions::margin_pool::process_configure_pool(client, options).await
        }
        MarginPoolCommand::CollectFees => {
            actions::margin_pool::process_collect_pool_fees(client).await
        }
        MarginPoolCommand::TransferLoan {
            source,
            target,
            token,
            amount,
        } => {
            actions::margin_pool::process_transfer_loan(client, source, target, token, amount).await
        }
        MarginPoolCommand::List => actions::margin_pool::process_list_pools(client).await,
        MarginPoolCommand::Show { token } => {
            actions::margin_pool::process_show_pool(client, token).await
        }
    }
}

async fn run_bonds_command(client: &Client, command: BondsCommand) -> Result<Plan> {
    match command {
        BondsCommand::CreateMarket(params) => {
            actions::bonds::process_create_bond_market(client, params).await
        }
    }
}

async fn run_test_command(client: &Client, command: TestCommand) -> Result<Plan> {
    match command {
        TestCommand::InitEnv { config_path } => {
            actions::test::process_init_env(client, config_path).await
        }

        TestCommand::GenerateAppConfig {
            config_path,
            output,
        } => actions::test::process_generate_app_config(client, config_path, output).await,
    }
}
