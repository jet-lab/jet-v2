use std::{
    borrow::Cow,
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime},
};

use anchor_lang::{prelude::Pubkey, AccountDeserialize};
use anyhow::{anyhow, bail, Context, Error, Result};

use dialoguer::Confirm;
use indicatif::{MultiProgress, ProgressBar};
use jet_rpc::solana_rpc_api::{AsyncSigner, SolanaRpcClient};
use solana_cli_config::{Config as SolanaConfig, CONFIG_FILE as SOLANA_CONFIG_FILE};
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, hash::Hash, instruction::Instruction,
    program_pack::Pack, signer::Signer, transaction::Transaction,
};

const MAINNET_HASH: &str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d";
const DEVNET_HASH: &str = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG";

#[derive(Debug, Eq, PartialEq)]
pub enum NetworkKind {
    Mainnet,
    Devnet,
    Other,
}

pub struct ClientConfig {
    /// If true, transactions will be simulated but not actually submitted.
    dry_run: bool,

    /// If true, will not ask user to confirm before submitting transactions
    no_confirm: bool,

    /// The user wallet
    signer: Option<AsyncSigner>,

    /// Set compute budget to
    compute_budget: Option<u32>,
}

impl ClientConfig {
    pub fn new(
        dry_run: bool,
        no_confirm: bool,
        signer_path: Option<String>,
        compute_budget: Option<u32>,
    ) -> Result<ClientConfig> {
        let solana_config =
            SolanaConfig::load(SOLANA_CONFIG_FILE.as_ref().unwrap()).unwrap_or_default();
        let mut remote_wallet_manager = None;

        let signer = solana_clap_utils::keypair::signer_from_path(
            &Default::default(),
            signer_path.as_ref().unwrap_or(&solana_config.keypair_path),
            "wallet",
            &mut remote_wallet_manager,
        )
        .map(Arc::from)
        .map(|s| AsyncSigner::new(s))
        .ok();

        Ok(ClientConfig {
            dry_run,
            no_confirm,
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

    /// The rpc connection for this client
    pub rpc: Arc<dyn SolanaRpcClient>,

    /// The configuration for this client
    pub config: ClientConfig,
}

impl Client {
    pub async fn new(rpc: Arc<dyn SolanaRpcClient>, config: ClientConfig) -> Result<Self> {
        let recent_blockhash = rpc.get_latest_blockhash().await?;
        let network_kind = Self::get_network_kind(rpc.clone()).await?;

        println!("connected to {:?}", &network_kind);

        Ok(Client {
            recent_blockhash,
            network_kind,
            rpc,
            config,
        })
    }

    /// Get the current network type
    async fn get_network_kind(rpc: Arc<dyn SolanaRpcClient>) -> Result<NetworkKind> {
        let mainnet_hash = Hash::from_str(MAINNET_HASH).unwrap();
        let devnet_hash = Hash::from_str(DEVNET_HASH).unwrap();

        let network_hash = rpc.get_genesis_hash().await?;

        Ok(match network_hash {
            hash if hash == mainnet_hash => NetworkKind::Mainnet,
            hash if hash == devnet_hash => NetworkKind::Devnet,
            _ => NetworkKind::Other,
        })
    }

    pub fn signer(&self) -> Result<Pubkey> {
        match &self.config.signer {
            Some(signer) => Ok(signer.pubkey()),
            None => bail!("no wallet/signer configured"),
        }
    }

    /// Check if an account exists (has a balance)
    pub async fn account_exists(&self, address: &Pubkey) -> Result<bool> {
        Ok(self.rpc.get_balance(address).await? > 0)
    }

    /// Deserialize an anchor compatible account
    pub async fn read_anchor_account<T: AccountDeserialize>(&self, address: &Pubkey) -> Result<T> {
        let account_data = self
            .rpc
            .get_account(address)
            .await?
            .ok_or_else(|| Error::msg("failed to fetch account: {address}"))?
            .data;
        Ok(AccountDeserialize::try_deserialize(&mut &account_data[..])?)
    }

    /// Read a mint account
    pub async fn read_mint(&self, address: &Pubkey) -> Result<spl_token::state::Mint> {
        let account_data = self
            .rpc
            .get_account(address)
            .await
            .with_context(|| format!("while retrieving mint data for {address}"))?
            .ok_or_else(|| Error::msg("failed to fetch account: {address}"))?
            .data;
        Ok(Pack::unpack(&account_data)?)
    }

    pub fn plan(&self) -> Result<PlanBuilder> {
        if self.config.signer.is_none() {
            bail!("no wallet/signer configured");
        }

        Ok(PlanBuilder {
            entries: Vec::new(),
            client: self,
        })
    }

    /// Execute a plan
    pub async fn execute(&self, mut plan: Plan) -> Result<()> {
        if plan.is_empty() {
            return Ok(());
        }

        println!("planning to submit {} transactions:", plan.len());

        let signer = match &self.config.signer {
            Some(signer) => signer,
            None => bail!("no wallet/signer configured"),
        };

        for entry in &mut plan {
            entry
                .transaction
                .partial_sign(&[&*signer], self.recent_blockhash);
        }

        for (i, entry) in plan.iter().enumerate() {
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

        let mut ui_progress_group = ProgressTracker::new();
        let ui_progress_tx = plan
            .iter()
            .map(|_| ui_progress_group.add_line("in queue"))
            .collect::<Vec<_>>();

        std::thread::spawn(move || ui_progress_group.join().unwrap());

        for (entry, ui_progress_bar) in plan.iter().zip(ui_progress_tx.into_iter()) {
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

        println!("submitted {} transactions", plan.len());
        Ok(())
    }

    async fn simulate_transaction(
        &self,
        transaction: &Transaction,
        ui_progress: Spinner,
    ) -> Result<()> {
        ui_progress.set_message("simulating");

        let result = self.rpc.simulate_transaction(transaction).await?;

        if let Some(e) = result.err {
            ui_progress.abandon_with_message("failed");

            bail!(
                "simulation failed '{e}': {:#?}",
                result.logs.unwrap_or_default()
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
                .rpc
                .send_transaction(transaction)
                .await
                .map_err(|e| anyhow!("error sending transaction: {e:?}"))?;

            ui_progress.set_message(format!("confirming {signature}"));

            let start_time = SystemTime::now();
            let max_wait_time = Duration::from_secs(90);

            loop {
                let status = self.rpc.get_signature_statuses(&[signature]).await?[0].clone();

                if SystemTime::now().duration_since(start_time).unwrap() > max_wait_time {
                    break;
                }

                match status {
                    None => tokio::time::sleep(Duration::from_millis(100)).await,
                    Some(status) => {
                        if let Some(e) = status.err {
                            ui_progress.abandon_with_message(format!("failed {signature}: {e:?}"));
                            bail!("transaction failed");
                        }
                        ui_progress.finish_with_message(format!("confirmed {signature}"));
                        return Ok(());
                    }
                }
            }
        }
    }
}

pub type Plan = Vec<TransactionEntry>;

pub struct PlanBuilder<'client> {
    entries: Vec<TransactionEntry>,
    client: &'client Client,
}

impl<'client> PlanBuilder<'client> {
    /// Add instructions to the plan, as a single transaction
    pub fn instructions<'a>(
        mut self,
        signers: impl IntoIterator<Item = &'a dyn Signer>,
        steps: impl IntoIterator<Item = impl AsRef<str>>,
        instructions: impl IntoIterator<Item = Instruction>,
    ) -> Self {
        let signers = signers.into_iter().collect::<Vec<_>>();
        let mut ix_list = match self.client.config.compute_budget {
            None => vec![],
            Some(budget) => {
                vec![ComputeBudgetInstruction::set_compute_unit_limit(budget)]
            }
        };

        ix_list.extend(instructions);
        let steps = steps.into_iter().map(|s| s.as_ref().to_owned()).collect();

        let mut transaction = Transaction::new_with_payer(
            &ix_list,
            Some(&self.client.config.signer.as_ref().unwrap().pubkey()),
        );
        transaction.partial_sign(&signers, self.client.recent_blockhash);

        self.entries.push(TransactionEntry { steps, transaction });

        self
    }

    pub fn build(self) -> Plan {
        self.entries
    }
}

pub struct TransactionEntry {
    pub steps: Vec<String>,
    pub transaction: Transaction,
}

struct ProgressTracker {
    container: MultiProgress,
    next_index: usize,
}

impl ProgressTracker {
    fn new() -> Self {
        ProgressTracker {
            container: MultiProgress::new(),
            next_index: 0,
        }
    }

    fn add_line(&mut self, msg: impl Into<Cow<'static, str>>) -> Spinner {
        let mut spinner = Spinner::new(self.next_index, msg);

        self.next_index += 1;
        spinner.bar = self.container.add(spinner.bar);

        spinner
    }

    fn join(&self) -> Result<()> {
        Ok(self.container.join()?)
    }
}

#[derive(Debug)]
pub struct Spinner {
    bar: ProgressBar,
    idx: usize,
}

impl Spinner {
    pub fn new(idx: usize, msg: impl Into<Cow<'static, str>>) -> Self {
        let bar = ProgressBar::new_spinner();

        bar.enable_steady_tick(100);
        bar.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner:.blue} {msg}")
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );

        let instance = Self { idx, bar };
        instance.set_message(msg);

        instance
    }

    pub fn set_message(&self, msg: impl Into<Cow<'static, str>>) {
        self.bar.set_message(self.style_msg(msg));
    }

    pub fn finish_with_message(&self, msg: impl Into<Cow<'static, str>>) {
        self.bar
            .set_style(indicatif::ProgressStyle::default_spinner().template("✅ {msg}"));
        self.bar.finish_with_message(self.style_msg(msg));
    }

    pub fn abandon_with_message(&self, msg: impl Into<Cow<'static, str>>) {
        self.bar
            .set_style(indicatif::ProgressStyle::default_spinner().template(" {msg}"));
        self.bar.abandon_with_message(self.style_msg(msg));
    }

    fn style_msg(&self, msg: impl Into<Cow<'static, str>>) -> String {
        format!("[#{}] {}", self.idx, msg.into())
    }
}
