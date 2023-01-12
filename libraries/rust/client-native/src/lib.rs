use async_trait::async_trait;
use std::{fmt::Debug, sync::Arc};

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    account::Account,
    hash::Hash,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::VersionedTransaction,
};

use jet_client::{ClientError, ClientResult, JetClient};
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use jet_solana_client::NetworkUserInterface;

pub type JetSimulationClientResult<T> = ClientResult<SimulationClient, T>;
pub type JetSimulationClientError = ClientError<SimulationClient>;
pub type JetSimulationClient = JetClient<SimulationClient>;
pub type JetSolanaClient = JetClient<SolanaClient>;

#[derive(Clone)]
pub struct SimulationClient {
    rpc: Arc<dyn SolanaRpcClient>,
    signer: Arc<dyn Signer + Send + Sync>,
}

impl Debug for SimulationClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SimulationClient")
            .field(&self.rpc.payer().pubkey())
            .finish()
    }
}

impl SimulationClient {
    pub fn new(rpc: Arc<dyn SolanaRpcClient>, wallet: Option<Keypair>) -> Self {
        Self {
            signer: Arc::new(
                wallet.unwrap_or_else(|| Keypair::from_bytes(&rpc.payer().to_bytes()).unwrap()),
            ),
            rpc,
        }
    }
}

#[async_trait(?Send)]
impl NetworkUserInterface for SimulationClient {
    type Error = anyhow::Error;

    fn signer(&self) -> Pubkey {
        self.signer.pubkey()
    }

    fn get_current_time(&self) -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    async fn get_genesis_hash(&self) -> Result<Hash, Self::Error> {
        self.rpc.get_genesis_hash().await
    }

    async fn get_latest_blockhash(&self) -> Result<Hash, Self::Error> {
        self.rpc.get_latest_blockhash().await
    }

    async fn get_accounts(
        &self,
        addresses: &[Pubkey],
    ) -> Result<Vec<Option<Account>>, Self::Error> {
        let accounts = self.rpc.get_multiple_accounts(addresses).await?;

        Ok(accounts
            .into_iter()
            .map(|account| match account {
                None => None,
                Some(account) if account.lamports == 0 => None,
                Some(a) => Some(a),
            })
            .collect())
    }

    async fn send_ordered(
        &self,
        transactions: &[VersionedTransaction],
    ) -> (Vec<Signature>, Option<Self::Error>) {
        let mut sigs = vec![];
        let mut error = None;

        for tx in transactions {
            // FIXME: how to use versioned tx?
            let mut legacy_tx = tx.clone().into_legacy_transaction().unwrap();
            let recent_blockhash = match self.rpc.get_latest_blockhash().await {
                Ok(hash) => hash,
                Err(e) => {
                    error = Some(e);
                    break;
                }
            };

            legacy_tx.partial_sign(&[self.signer.as_ref() as &dyn Signer], recent_blockhash);

            let sig = match self.rpc.send_and_confirm_transaction(&legacy_tx).await {
                Ok(sig) => sig,
                Err(e) => {
                    error = Some(e);
                    break;
                }
            };

            sigs.push(sig);
        }

        (sigs, error)
    }

    async fn send_unordered(
        &self,
        transactions: &[VersionedTransaction],
        recent_blockhash: Option<Hash>,
    ) -> Vec<Result<Signature, Self::Error>> {
        futures::future::join_all(transactions.iter().map(|tx| async {
            // FIXME: support versioned tx in simulator
            let mut legacy_tx = tx.clone().into_legacy_transaction().unwrap();
            let recent_blockhash = match recent_blockhash {
                None => self.rpc.get_latest_blockhash().await?,
                Some(hash) => hash,
            };

            legacy_tx.partial_sign(&[self.signer.as_ref() as &dyn Signer], recent_blockhash);
            self.rpc.send_and_confirm_transaction(&legacy_tx).await
        }))
        .await
        .into_iter()
        .collect()
    }
}

#[derive(Clone)]
pub struct SolanaClient {
    rpc: Arc<RpcClient>,
    signer: Arc<dyn Signer>,
}

impl SolanaClient {
    pub fn new(rpc: RpcClient, signer: Arc<dyn Signer>) -> Self {
        Self {
            rpc: Arc::new(rpc),
            signer,
        }
    }
}

impl Debug for SolanaClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SolanaClient")
            .field(&self.signer.pubkey())
            .finish()
    }
}


#[async_trait(?Send)]
impl NetworkUserInterface for SolanaClient {
    type Error = solana_client::client_error::ClientError;

    fn signer(&self) -> Pubkey {
        self.signer.pubkey()
    }

    fn get_current_time(&self) -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    async fn get_genesis_hash(&self) -> Result<Hash, Self::Error> {
        self.rpc.get_genesis_hash().await
    }

    async fn get_latest_blockhash(&self) -> Result<Hash, Self::Error> {
        self.rpc.get_latest_blockhash().await
    }

    async fn get_accounts(
        &self,
        addresses: &[Pubkey],
    ) -> Result<Vec<Option<Account>>, Self::Error> {
        self.rpc.get_multiple_accounts(addresses).await
    }

    async fn send_ordered(
        &self,
        transactions: &[VersionedTransaction],
    ) -> (Vec<Signature>, Option<Self::Error>) {
        let mut sigs = vec![];
        let mut error = None;

        for tx in transactions {
            // FIXME: how to use versioned tx?
            let mut legacy_tx = tx.clone().into_legacy_transaction().unwrap();
            let recent_blockhash = match self.rpc.get_latest_blockhash().await {
                Ok(hash) => hash,
                Err(e) => {
                    error = Some(e);
                    break;
                }
            };

            legacy_tx.partial_sign(&[self.signer.as_ref() as &dyn Signer], recent_blockhash);

            let sig = match self.rpc.send_and_confirm_transaction(&legacy_tx).await {
                Ok(sig) => sig,
                Err(e) => {
                    error = Some(e);
                    break;
                }
            };

            sigs.push(sig);
        }

        (sigs, error)
    }

    async fn send_unordered(
        &self,
        transactions: &[VersionedTransaction],
        recent_blockhash: Option<Hash>,
    ) -> Vec<Result<Signature, Self::Error>> {
        futures::future::join_all(transactions.iter().map(|tx| async {
            // FIXME: how to use versioned tx?
            let mut legacy_tx = tx.clone().into_legacy_transaction().unwrap();
            let recent_blockhash = match recent_blockhash {
                None => self.rpc.get_latest_blockhash().await?,
                Some(hash) => hash,
            };

            legacy_tx.partial_sign(&[self.signer.as_ref() as &dyn Signer], recent_blockhash);
            self.rpc.send_and_confirm_transaction(&legacy_tx).await
        }))
        .await
        .into_iter()
        .collect()
    }
}
