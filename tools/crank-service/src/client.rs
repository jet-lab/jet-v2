use std::sync::Arc;

use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{instruction::Instruction, signature::Signature, message::Message, transaction::Transaction, signer::Signer};

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
