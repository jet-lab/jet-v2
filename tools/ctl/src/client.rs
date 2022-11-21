use std::{
    borrow::Cow,
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime},
};

use anchor_lang::{prelude::Pubkey, AccountDeserialize};
use anyhow::{anyhow, bail, Context, Result};

use dialoguer::Confirm;
use indicatif::{MultiProgress, ProgressBar};
use solana_cli_config::{Config as SolanaConfig, CONFIG_FILE as SOLANA_CONFIG_FILE};
use solana_client::{
    client_error::ClientErrorKind,
    nonblocking::rpc_client::RpcClient,
    rpc_config::RpcSendTransactionConfig,
    rpc_request::{RpcError, RpcResponseErrorData},
};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    compute_budget::ComputeBudgetInstruction,
    hash::Hash,
    instruction::Instruction,
    program_pack::Pack,
    signer::Signer,
    transaction::Transaction,
};

const MAINNET_HASH: &str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d";
const DEVNET_HASH: &str = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG";

#[derive(Debug, Eq, PartialEq)]
pub enum NetworkKind {
    Mainnet,
    Devnet,
    Localnet,
}

pub struct ClientConfig {
    /// If true, transactions will be simulated but not actually submitted.
    dry_run: bool,

    /// If true, will not ask user to confirm before submitting transactions
    no_confirm: bool,

    /// The solana rpc client
    rpc_client: RpcClient,

    /// The user wallet
    signer: Option<Arc<dyn Signer>>,

    /// Set compute budget to
    compute_budget: Option<u32>,
}

impl ClientConfig {
    pub fn new(
        dry_run: bool,
        no_confirm: bool,
        signer_path: Option<String>,
        rpc_endpoint: Option<String>,
        compute_budget: Option<u32>,
    ) -> Result<ClientConfig> {
        let solana_config =
            SolanaConfig::load(SOLANA_CONFIG_FILE.as_ref().unwrap()).unwrap_or_default();
        let rpc_url = rpc_endpoint.unwrap_or(solana_config.json_rpc_url);
        let rpc_client = RpcClient::new(rpc_url);
        let mut remote_wallet_manager = None;

        let signer = solana_clap_utils::keypair::signer_from_path(
            &Default::default(),
            signer_path.as_ref().unwrap_or(&solana_config.keypair_path),
            "wallet",
            &mut remote_wallet_manager,
        )
        .map(Arc::from)
        .ok();

        Ok(ClientConfig {
            dry_run,
            no_confirm,
            rpc_client,
            signer,
            compute_budget,
        })
    }
}

/// A client for interacting with the solana network
pub struct Client {
    /// A recent blockhash from the network
    pub recent_blockhash: Hash,

    /// The network type this client is connected to
    pub network_kind: NetworkKind,

    /// The configuration for this client
    pub config: ClientConfig,
}

impl Client {
    pub async fn new(config: ClientConfig) -> Result<Self> {
        let recent_blockhash = config.rpc_client.get_latest_blockhash().await?;
        let network_kind = Self::get_network_kind(&config.rpc_client).await?;

        println!("connected to {:?}", &network_kind);

        Ok(Client {
            recent_blockhash,
            network_kind,
            config,
        })
    }

    /// Get the current network type
    async fn get_network_kind(rpc: &RpcClient) -> Result<NetworkKind> {
        let mainnet_hash = Hash::from_str(MAINNET_HASH).unwrap();
        let devnet_hash = Hash::from_str(DEVNET_HASH).unwrap();

        let network_hash = rpc.get_genesis_hash().await?;

        Ok(match network_hash {
            hash if hash == mainnet_hash => NetworkKind::Mainnet,
            hash if hash == devnet_hash => NetworkKind::Devnet,
            _ => NetworkKind::Localnet,
        })
    }

    pub fn rpc(&self) -> &RpcClient {
        &self.config.rpc_client
    }

    pub fn signer(&self) -> Result<Pubkey> {
        match &self.config.signer {
            Some(signer) => Ok(signer.pubkey()),
            None => bail!("no wallet/signer configured"),
        }
    }

    pub fn sign(&self, tx: &mut Transaction) -> Result<()> {
        match &self.config.signer {
            None => bail!("no wallet/signer configured"),
            Some(signer) => {
                tx.partial_sign(&[&**signer], self.recent_blockhash);
                Ok(())
            }
        }
    }

    /// Check if an account exists (has a balance)
    pub async fn account_exists(&self, address: &Pubkey) -> Result<bool> {
        Ok(self.config.rpc_client.get_balance(address).await? > 0)
    }

    /// Deserialize an anchor compatible account
    pub async fn read_anchor_account<T: AccountDeserialize>(&self, address: &Pubkey) -> Result<T> {
        let account_data = self.get_account_data(address).await?;
        Ok(AccountDeserialize::try_deserialize(&mut &account_data[..])?)
    }

    /// Read a mint account
    pub async fn read_mint(&self, address: &Pubkey) -> Result<spl_token::state::Mint> {
        let account_data = self
            .get_account_data(address)
            .await
            .with_context(|| format!("while retrieving mint data for {address}"))?;
        Ok(Pack::unpack(&account_data)?)
    }

    pub async fn read_token_account(&self, address: &Pubkey) -> Result<spl_token::state::Account> {
        let account_data = self
            .get_account_data(address)
            .await
            .with_context(|| format!("while retrieving token account data for {address}"))?;
        Ok(Pack::unpack(&account_data)?)
    }

    pub fn plan(&self) -> Result<PlanBuilder> {
        if self.config.signer.is_none() {
            bail!("no wallet/signer configured");
        }

        Ok(PlanBuilder {
            entries: Vec::new(),
            client: self,
            unordered: false,
        })
    }

    /// Execute a plan
    pub async fn execute(&self, plan: Plan) -> Result<()> {
        if plan.entries.is_empty() {
            return Ok(());
        }

        if self.config.dry_run {
            println!("this is a dry run");
        }

        println!("planning to submit {} transactions:", plan.entries.len());

        let signer = match &self.config.signer {
            Some(signer) => signer,
            None => bail!("no wallet/signer configured"),
        };

        for (i, entry) in plan.entries.iter().enumerate() {
            let tx_size = entry.transaction.message().serialize().len();
            println!("\t transaction #{i} (size {tx_size}):");

            for (j, step) in entry.steps.iter().enumerate() {
                println!("\t\t [{j}] {step}");
            }
        }

        println!();

        if !self.config.no_confirm {
            let confirmed = Confirm::new()
                .with_prompt("Submit these transactions?")
                .default(false)
                .interact()?;

            if !confirmed {
                bail!("submission aborted");
            }
        }

        let mut ui_progress_group = ProgressTracker::new(self.config.no_confirm);

        #[allow(clippy::needless_collect)]
        let ui_progress_tx = plan
            .entries
            .iter()
            .map(|_| ui_progress_group.add_line("in queue"))
            .collect::<Vec<_>>();

        if !self.config.no_confirm {
            std::thread::spawn(move || ui_progress_group.join().unwrap());
        }

        let tx_count = plan.entries.len();

        for (mut entry, ui_progress_bar) in plan.entries.into_iter().zip(ui_progress_tx.into_iter())
        {
            let recent_blockhash = self.rpc().get_latest_blockhash().await?;
            entry
                .transaction
                .partial_sign(&entry.signers, recent_blockhash);
            entry
                .transaction
                .partial_sign(&[&**signer], recent_blockhash);

            match self.config.dry_run {
                false => {
                    self.submit_transaction(&entry.transaction, ui_progress_bar)
                        .await?
                }
                true => {
                    self.simulate_transaction(&entry.transaction, ui_progress_bar)
                        .await?
                }
            }
        }

        println!("submitted {} transactions", tx_count);
        Ok(())
    }

    async fn simulate_transaction(
        &self,
        transaction: &Transaction,
        ui_progress: Spinner,
    ) -> Result<()> {
        ui_progress.set_message("simulating");

        let result = self.rpc().simulate_transaction(transaction).await?;

        if let Some(e) = result.value.err {
            ui_progress.abandon_with_message("failed");

            bail!(
                "simulation failed '{e}': {:#?}",
                result.value.logs.unwrap_or_default()
            );
        }

        ui_progress.finish_with_message("success");
        Ok(())
    }

    async fn submit_transaction(
        &self,
        transaction: &Transaction,
        ui_progress: Spinner,
    ) -> Result<()> {
        loop {
            ui_progress.set_message("submitting");

            let signature = self
                .config
                .rpc_client
                .send_transaction_with_config(
                    transaction,
                    RpcSendTransactionConfig {
                        preflight_commitment: Some(CommitmentLevel::Processed),
                        ..Default::default()
                    },
                )
                .await
                .map_err(|e| match e.kind {
                    ClientErrorKind::RpcError(RpcError::RpcResponseError {
                        data, message, ..
                    }) => {
                        if let RpcResponseErrorData::SendTransactionPreflightFailure(result) = data
                        {
                            anyhow!(
                                "preflight simulation failed: {:#?}",
                                result.logs.unwrap_or_default()
                            )
                        } else {
                            anyhow!("RPC error: {message}")
                        }
                    }

                    _ => anyhow!("error sending transaction: {e:?}"),
                })?;

            ui_progress.set_message(format!("confirming {signature}"));

            let start_time = SystemTime::now();
            let max_wait_time = Duration::from_secs(90);

            loop {
                let status = self
                    .config
                    .rpc_client
                    .get_signature_status_with_commitment(
                        &signature,
                        CommitmentConfig {
                            commitment: CommitmentLevel::Processed,
                        },
                    )
                    .await?;

                if SystemTime::now().duration_since(start_time).unwrap() > max_wait_time {
                    break;
                }

                match status {
                    None => tokio::time::sleep(Duration::from_millis(100)).await,
                    Some(Ok(())) => {
                        ui_progress.finish_with_message(format!("confirmed {signature}"));
                        return Ok(());
                    }
                    Some(Err(e)) => {
                        ui_progress.abandon_with_message(format!("failed {signature}: {e:?}"));
                        bail!("transaction failed");
                    }
                }
            }
        }
    }

    async fn get_account_data(&self, address: &Pubkey) -> Result<Vec<u8>> {
        Ok(self
            .rpc()
            .get_account_with_commitment(
                address,
                CommitmentConfig {
                    commitment: CommitmentLevel::Processed,
                },
            )
            .await?
            .value
            .ok_or_else(|| anyhow::format_err!("could not find account {}", address))?
            .data)
    }
}

#[derive(Default)]
pub struct Plan {
    pub entries: Vec<TransactionEntry>,
    pub unordered: bool,
}

pub struct PlanBuilder<'client> {
    entries: Vec<TransactionEntry>,
    unordered: bool,
    client: &'client Client,
}

impl<'client> PlanBuilder<'client> {
    /// Add instructions to the plan, as a single transaction
    pub fn instructions(
        mut self,
        signers: impl IntoIterator<Item = Box<dyn Signer>>,
        steps: impl IntoIterator<Item = impl AsRef<str>>,
        instructions: impl IntoIterator<Item = Instruction>,
    ) -> Self {
        let mut ix_list = match self.client.config.compute_budget {
            None => vec![],
            Some(budget) => {
                vec![ComputeBudgetInstruction::set_compute_unit_limit(budget)]
            }
        };

        ix_list.extend(instructions);

        if ix_list.is_empty() {
            return self;
        }

        let steps = steps.into_iter().map(|s| s.as_ref().to_owned()).collect();

        let transaction = Transaction::new_with_payer(
            &ix_list,
            Some(&self.client.config.signer.as_ref().unwrap().pubkey()),
        );

        self.entries.push(TransactionEntry {
            steps,
            transaction,
            signers: signers.into_iter().collect(),
        });

        self
    }

    pub fn unordered(mut self) -> Self {
        self.unordered = true;
        self
    }

    pub fn build(self) -> Plan {
        Plan {
            entries: self.entries,
            unordered: self.unordered,
        }
    }
}

pub struct TransactionEntry {
    pub steps: Vec<String>,
    pub transaction: Transaction,
    pub signers: Vec<Box<dyn Signer>>,
}

struct ProgressTracker {
    container: MultiProgress,
    next_index: usize,
    disabled: bool,
}

impl ProgressTracker {
    fn new(disabled: bool) -> Self {
        ProgressTracker {
            container: MultiProgress::new(),
            next_index: 0,
            disabled,
        }
    }

    fn add_line(&mut self, msg: impl Into<Cow<'static, str>>) -> Spinner {
        let mut spinner = Spinner::new(self.disabled, self.next_index, msg);

        self.next_index += 1;
        spinner.bar = spinner.bar.map(|bar| self.container.add(bar));

        spinner
    }

    fn join(&self) -> Result<()> {
        Ok(self.container.join()?)
    }
}

#[derive(Debug)]
pub struct Spinner {
    bar: Option<ProgressBar>,
    idx: usize,
}

impl Spinner {
    pub fn new(disabled: bool, idx: usize, msg: impl Into<Cow<'static, str>>) -> Self {
        if disabled {
            return Self { idx, bar: None };
        }

        let bar = ProgressBar::new_spinner();

        bar.enable_steady_tick(100);
        bar.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner:.blue} {msg}")
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );

        let instance = Self {
            idx,
            bar: Some(bar),
        };
        instance.set_message(msg);

        instance
    }

    pub fn set_message(&self, msg: impl Into<Cow<'static, str>>) {
        if let Some(bar) = &self.bar {
            bar.set_message(self.style_msg(msg));
        }
    }

    pub fn finish_with_message(&self, msg: impl Into<Cow<'static, str>>) {
        if let Some(bar) = &self.bar {
            bar.set_style(indicatif::ProgressStyle::default_spinner().template("✅ {msg}"));
            bar.finish_with_message(self.style_msg(msg));
        }
    }

    pub fn abandon_with_message(&self, msg: impl Into<Cow<'static, str>>) {
        if let Some(bar) = &self.bar {
            bar.set_style(indicatif::ProgressStyle::default_spinner().template(" {msg}"));
            bar.abandon_with_message(self.style_msg(msg));
        }
    }

    fn style_msg(&self, msg: impl Into<Cow<'static, str>>) -> String {
        format!("[#{}] {}", self.idx, msg.into())
    }
}
