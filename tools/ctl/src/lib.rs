use std::path::PathBuf;

use actions::{
    fixed_term::{FixedTermDisplayCmd, MarketParameters},
    margin_pool::ConfigurePoolCliOptions, airdrop::AirdropCommand,
};
use anchor_lang::prelude::Pubkey;
use anyhow::{bail, Result};
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
mod ix_inspectors;
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

    /// The relevant airspace to use
    #[clap(global = true, long, env = "JET_AIRSPACE_ID")]
    pub airspace: Option<Pubkey>,

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
    #[clap(global = true, long, short = 'k', env = "SIGNER_PATH")]
    pub signer_path: Option<String>,

    /// The network endpoint to use
    #[clap(global = true, long, short = 'u', env = "RPC_URL")]
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

    /// Add refresher permissions
    AddRefresher {
        /// The liquidator's address
        #[serde_as(as = "DisplayFromStr")]
        account: Pubkey,
    },

    /// Remove refresher permissions
    RemoveRefresher {
        /// The liquidator's address
        #[serde_as(as = "DisplayFromStr")]
        account: Pubkey,
    },

    /// Update the metadata for existing positions after an update to the global token config
    RefreshPositionMd,

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

    /// Read the current state of the margin config for a token
    ReadTokenConfig {
        /// The token or config address
        address: Pubkey,
    },

    /// Configure the airspace for accounts that are missing the airspace field
    ConfigureAccountAirspaces,
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
#[serde(tag = "fixed-term-market-action")]
pub enum FixedTermCommand {
    /// Create a new fixed term market
    CreateMarket(MarketParameters),

    /// Recover unintialized account rent
    RecoverUninitialized { recipient: Pubkey },

    /// Fetch, deserialize and display FixedTerm related accounts
    #[clap(subcommand)]
    Display(FixedTermDisplayCmd),
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

        /// The address to use as a default lookup registry, supplied directly
        #[clap(long)]
        override_lookup_authority: Option<Pubkey>,
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

    /// Fixed term market management
    Fixed {
        #[clap(subcommand)]
        subcmd: FixedTermCommand,
    },

    /// Airdrop management
    Airdrop {
        #[clap(subcommand)]
        subcmd: AirdropCommand
    }
}

pub async fn run(opts: CliOpts) -> Result<()> {
    let _ = env_logger::builder().is_test(false).try_init();

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
    let skip_proposal_conversion = matches!(&opts.command, Command::Apply { .. });

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
            actions::apply::process_apply(
                &client,
                config_path,
                opts.target_proposal,
                opts.target_proposal_option,
            )
            .await?
        }
        Command::GenerateAppConfig {
            config_dir,
            output,
            override_lookup_authority,
        } => {
            actions::global::process_generate_app_config(
                &client,
                &config_dir,
                &output,
                override_lookup_authority,
            )
            .await?
        }
        Command::Proposals { subcmd } => run_proposals_command(&client, subcmd).await?,
        Command::CreateAuthority => actions::global::process_create_authority(&client).await?,
        Command::CheckMetadata { address } => {
            actions::global::process_check_metadata(&client, address).await?
        }
        Command::Margin { subcmd } => {
            let airspace = match opts.airspace {
                Some(airspace) => airspace,
                None => {
                    bail!("the --airspace option is required for margin account commands")
                }
            };

            run_margin_command(&client, subcmd, airspace).await?
        }
        Command::MarginPool { subcmd } => run_margin_pool_command(&client, subcmd).await?,
        Command::Fixed { subcmd } => run_fixed_command(&client, subcmd).await?,
        Command::Airdrop { subcmd } => actions::airdrop::run_command(&client, subcmd).await?
    };

    if let Some(proposal_id) = opts.target_proposal {
        if !skip_proposal_conversion {
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

async fn run_margin_command(
    client: &Client,
    command: MarginCommand,
    airspace: Pubkey,
) -> Result<Plan> {
    match command {
        MarginCommand::RegisterAdapter { address } => {
            actions::margin::process_register_adapter(client, airspace, address).await
        }
        MarginCommand::AddLiquidator { liquidator } => {
            actions::margin::process_set_liquidator(client, airspace, liquidator, true).await
        }
        MarginCommand::RemoveLiquidator { liquidator } => {
            actions::margin::process_set_liquidator(client, airspace, liquidator, false).await
        }
        MarginCommand::AddRefresher { account } => {
            actions::margin::process_set_refresher_permission(client, airspace, account, true).await
        }
        MarginCommand::RemoveRefresher { account } => {
            actions::margin::process_set_refresher_permission(client, airspace, account, false)
                .await
        }
        MarginCommand::RefreshPositionMd => {
            actions::margin::process_refresh_metadata(client, airspace).await
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
            actions::margin::process_list_top_accounts(client, airspace, limit).await
        }
        MarginCommand::Inspect { addresses } => {
            actions::margin::process_inspect(client, addresses).await
        }
        MarginCommand::ReadTokenConfig { address } => {
            actions::margin::process_read_token_config(client, airspace, address).await
        }
        MarginCommand::ConfigureAccountAirspaces => {
            actions::margin::process_configure_account_airspaces(client).await
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

async fn run_fixed_command(client: &Client, command: FixedTermCommand) -> Result<Plan> {
    match command {
        FixedTermCommand::CreateMarket(params) => {
            actions::fixed_term::process_create_fixed_term_market(client, params).await
        }

        FixedTermCommand::RecoverUninitialized { recipient } => {
            actions::fixed_term::process_recover_uninitialized(client, recipient).await
        }

        FixedTermCommand::Display(cmd) => {
            actions::fixed_term::process_display_fixed_term_accounts(client, cmd).await?;

            Ok(Plan::default())
        }
    }
}
