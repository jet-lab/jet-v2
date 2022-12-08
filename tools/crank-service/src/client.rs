use std::sync::Arc;

use anyhow::Result;
use solana_cli_config::{Config as SolanaConfig, CONFIG_FILE as SOLANA_CONFIG_FILE};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction, message::Message, signature::Signature, signer::Signer,
    transaction::Transaction,
};

#[derive(Clone)]
pub struct Client {
    pub signer: AsyncSigner,
    pub conn: Arc<RpcClient>,
}

impl Client {
    pub fn new(signer_path: Option<String>, url: String) -> Result<Self> {
        Ok(Self {
            signer: AsyncSigner::load_from_path(signer_path)?,
            conn: Arc::new(RpcClient::new(url)),
        })
    }

    /// client only sends one intruction at a time due to synchrony needs
    pub async fn sign_send_ix(&self, ix: Instruction) -> Result<Signature> {
        let msg = Message::new(&[ix], Some(&self.signer.pubkey()));
        let mut tx = Transaction::new_unsigned(msg);

        let hash = self.conn.get_latest_blockhash().await?;
        tx.try_partial_sign(&[&self.signer], hash)?;
        self.conn
            .send_and_confirm_transaction(&tx)
            .await
            .map_err(anyhow::Error::from)
    }
}

#[derive(Clone)]
pub struct AsyncSigner(Arc<dyn Signer>);

impl AsyncSigner {
    pub fn load_from_path(path: Option<String>) -> Result<Self> {
        let default_signer = || {
            SolanaConfig::load(SOLANA_CONFIG_FILE.as_ref().unwrap())
                .unwrap_or_default()
                .keypair_path
        };
        solana_clap_utils::keypair::signer_from_path(
            &Default::default(),
            &path.unwrap_or_else(default_signer),
            "wallet",
            &mut None,
        )
        .map(Self::from)
        .map_err(|_| anyhow::Error::msg("failed to register signer from path"))
    }
}

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
