use std::{fs::read_to_string, sync::Arc};

use agnostic_orderbook::state::event_queue::EventQueue;
use anchor_lang::AccountDeserialize;
use anyhow::Result;
use clap::Parser;
use jet_margin_sdk::{
    bonds::{BondManager, BondsIxBuilder},
    ix_builder::{derive_airspace, test_service::derive_token_mint},
};
use jetctl::actions::test::{derive_bond_manager_from_duration_seed, TestEnvConfig};
use solana_cli_config::{Config as SolanaConfig, CONFIG_FILE as SOLANA_CONFIG_FILE};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction, message::Message, pubkey::Pubkey, signature::Signature,
    signer::Signer, transaction::Transaction,
};

static LOCALNET_URL: &str = "http://127.0.0.1:8899";

#[derive(Clone)]
pub struct AsyncSigner(Arc<dyn Signer>);

impl Signer for AsyncSigner {
    fn is_interactive(&self) -> bool {
        self.0.is_interactive()
    }
    fn pubkey(&self) -> solana_sdk::pubkey::Pubkey {
        self.0.pubkey()
    }
    fn try_pubkey(&self) -> Result<solana_sdk::pubkey::Pubkey, solana_sdk::signer::SignerError> {
        self.0.try_pubkey()
    }
    fn sign_message(&self, message: &[u8]) -> Signature {
        self.0.sign_message(message)
    }
    fn try_sign_message(
        &self,
        message: &[u8],
    ) -> Result<Signature, solana_sdk::signer::SignerError> {
        self.0.try_sign_message(message)
    }
}
unsafe impl Send for AsyncSigner {}
unsafe impl Sync for AsyncSigner {}

impl<T: Into<Arc<dyn Signer>>> From<T> for AsyncSigner {
    fn from(s: T) -> Self {
        Self(s.into())
    }
}

#[derive(Clone)]
pub struct Client {
    pub signer: AsyncSigner,
    pub conn: Arc<RpcClient>,
}

impl Client {
    pub fn new(signer: AsyncSigner, url: String) -> Self {
        Self {
            signer,
            conn: Arc::new(RpcClient::new(url)),
        }
    }

    pub fn sign_send_consume_ix(&self, ix: Instruction) -> Result<Signature> {
        let msg = Message::new(&[ix], Some(&self.signer.pubkey()));
        let mut tx = Transaction::new_unsigned(msg);

        tx.try_partial_sign(&[&self.signer], self.conn.get_latest_blockhash()?)?;
        self.conn
            .send_and_confirm_transaction(&tx)
            .map_err(anyhow::Error::from)
    }
}

#[derive(Parser, Debug)]
pub struct CliOpts {
    /// The filepath to the config file with market information
    #[clap(long, short = 'c')]
    pub config_path: String,

    /// The keypair to use for signing transactions
    #[clap(long, short = 'k')]
    pub keypair_path: Option<String>,

    /// The rpc endpoint
    /// Defaults to localhost
    #[clap(long, short = 'u')]
    pub url: Option<String>,
}

async fn run(opts: CliOpts) -> Result<()> {
    let client = Client::new(
        load_signer(opts.keypair_path)?,
        opts.url.unwrap_or_else(|| LOCALNET_URL.into()),
    );

    let cfg = read_config(&opts.config_path)?;
    for (asset, markets) in cfg {
        for market in markets {
            let c = client.clone();
            let a = asset.clone();

            std::thread::spawn(move || loop {
                let manager = {
                    let buf = c.conn.get_account_data(&market).unwrap();
                    BondManager::try_deserialize(&mut buf.as_slice()).unwrap()
                };
                let ix_builder = BondsIxBuilder::from(manager)
                    .with_crank(&c.signer.pubkey())
                    .with_payer(&c.signer.pubkey());
                let res = consume_events(c.clone(), ix_builder);
                println!(
                    "Market: {}_{} Result: {:#?}",
                    a, manager.borrow_duration, res
                );
            });
        }
    }
    Ok(())
}

fn consume_events(client: Client, ix_builder: BondsIxBuilder) -> Result<Signature> {
    // load event queue
    let mut eq_data = client.conn.get_account_data(&ix_builder.event_queue())?;
    let eq = EventQueue::from_buffer(
        &mut eq_data,
        agnostic_orderbook::state::AccountTag::EventQueue,
    )?;

    let consume = ix_builder.consume_events(eq)?;
    client.sign_send_consume_ix(consume)
}

fn read_config(path: &str) -> Result<Vec<(String, Vec<Pubkey>)>> {
    let cfg = read_to_string(path)?;
    Ok(toml::from_str::<TestEnvConfig>(&cfg)?
        .airspace
        .into_iter()
        .map(|a| {
            let markets = a
                .tokens
                .into_iter()
                .flat_map(|(_, c)| {
                    c.bond_markets.into_iter().map(|m| {
                        derive_bond_manager_from_duration_seed(
                            &derive_airspace(&a.name),
                            &derive_token_mint(&a.name),
                            m.borrow_duration,
                        )
                    })
                })
                .collect::<Vec<_>>();

            (a.name, markets)
        })
        .collect::<Vec<_>>())
}

fn load_signer(path: Option<String>) -> Result<AsyncSigner> {
    let solana_config =
        SolanaConfig::load(SOLANA_CONFIG_FILE.as_ref().unwrap()).unwrap_or_default();
    solana_clap_utils::keypair::signer_from_path(
        &Default::default(),
        path.as_ref().unwrap_or(&solana_config.keypair_path),
        "wallet",
        &mut None,
    )
    .map(Arc::from)
    .map(AsyncSigner::from)
    .map_err(|_| anyhow::Error::msg("failed to register signer from path"))
}

#[tokio::main]
async fn main() -> Result<()> {
    run(CliOpts::parse()).await
}
