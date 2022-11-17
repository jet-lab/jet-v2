use std::{fs::OpenOptions, io::Read, sync::Arc};

use agnostic_orderbook::state::event_queue::EventQueue;
use anyhow::Result;
use clap::Parser;
use jet_margin_sdk::bonds::BondsIxBuilder;
use jetctl::app_config::JetAppConfig;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    message::Message,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};

type Config = JetAppConfig;

#[derive(Clone)]
pub struct AsyncSigner(pub Arc<dyn Signer>);
unsafe impl Send for AsyncSigner {}
unsafe impl Sync for AsyncSigner {}

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
        let msg = Message::new(&[ix], Some(&self.signer.0.pubkey()));
        let mut tx = Transaction::new_unsigned(msg);
        let position = tx
            .get_signing_keypair_positions(&[self.signer.0.pubkey()])?
            .into_iter()
            .flatten()
            .next()
            .unwrap();
        tx.message.recent_blockhash = self.conn.get_latest_blockhash()?;
        tx.signatures[position] = self.signer.0.try_sign_message(&tx.message_data())?;

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
}

async fn run(opts: CliOpts) -> Result<()> {
    let cfg = load_config(&opts.config_path)?;
    let client = Client::new(load_signer(opts.keypair_path)?, cfg.url);

    let spaces = cfg.airspaces;
    for space in spaces {
        let markets = space
            .bond_markets
            .iter()
            .map(|(s, m)| (s.clone(), m.market_info))
            .collect::<Vec<_>>();
        for (market, manager) in markets {
            let c = client.clone();
            std::thread::spawn(move || loop {
                let ix_builder = BondsIxBuilder::from(manager)
                    .with_crank(&c.signer.0.pubkey())
                    .with_payer(&c.signer.0.pubkey());
                let res = consume_events(c.clone(), ix_builder);
                println!("Market: {} Result: {:#?}", market, res);
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

fn load_config(path: &str) -> Result<Config> {
    let mut s = String::new();
    OpenOptions::new()
        .read(true)
        .open(path)?
        .read_to_string(&mut s)?;
    serde_json::from_str(&s).map_err(anyhow::Error::from)
}

fn load_signer(path: Option<String>) -> Result<AsyncSigner> {
    let signer: Arc<dyn Signer> = match path {
        Some(p) => solana_clap_utils::keypair::signer_from_path(
            &Default::default(),
            p.as_ref(),
            "wallet",
            &mut None,
        )
        .map(Arc::from)
        .map_err(|_| anyhow::Error::msg("failed to register signer from path"))?,
        None => Arc::new(Keypair::new()),
    };

    Ok(AsyncSigner(signer))
}

#[tokio::main]
async fn main() -> Result<()> {
    run(CliOpts::parse()).await
}
