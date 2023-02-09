use std::{collections::VecDeque, error::Error as StdError, sync::Mutex};
use thiserror::Error;

use solana_sdk::{hash::Hash, instruction::Instruction, pubkey::Pubkey, signature::Signature};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};

use jet_solana_client::{ExtError, NetworkUserInterface};

use crate::{config::JetAppConfig, solana::transaction::ToTransaction, state::AccountStates};

pub type ClientResult<I, T> = std::result::Result<T, ClientError<I>>;

#[derive(Error)]
pub enum ClientError<I: NetworkUserInterface> {
    #[error("interface error")]
    Interface(I::Error),
    #[error("decode error: {0}")]
    Deserialize(Box<dyn StdError + Send + Sync>),
    #[error("error: {0}")]
    Unexpected(String),
}

impl<I: NetworkUserInterface> std::fmt::Debug for ClientError<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Interface(_) => write!(f, "interface error"),
            Self::Deserialize(e) => write!(f, "decode error: {}", e),
            Self::Unexpected(e) => write!(f, "error: {}", e),
        }
    }
}

impl<I: NetworkUserInterface> From<ExtError<I>> for ClientError<I> {
    fn from(e: ExtError<I>) -> Self {
        match e {
            ExtError::Interface(err) => Self::Interface(err),
            ExtError::Unpack { error, .. } => Self::Deserialize(Box::new(error)),
            ExtError::Deserialize { error, .. } => Self::Deserialize(Box::new(error)),
        }
    }
}

impl<I: NetworkUserInterface> From<bincode::Error> for ClientError<I> {
    fn from(err: bincode::Error) -> Self {
        Self::Unexpected(format!("unexpected encoding error: {err:?}"))
    }
}

/// Central object for client implementations, containing the global configuration and any
/// caching for account data.
pub struct ClientState<I> {
    pub(crate) network: I,
    state: AccountStates<I>,
    tx_log: Mutex<VecDeque<Signature>>,
}

impl<I: NetworkUserInterface> ClientState<I> {
    pub fn new(network: I, config: JetAppConfig, airspace: String) -> ClientResult<I, Self> {
        Ok(Self {
            state: AccountStates::new(network.clone(), config, airspace)?,
            tx_log: Mutex::new(VecDeque::new()),
            network,
        })
    }

    pub fn signer(&self) -> Pubkey {
        self.network.signer()
    }

    pub fn airspace(&self) -> Pubkey {
        self.state.config.airspace
    }

    pub fn state(&self) -> &AccountStates<I> {
        &self.state
    }

    pub async fn account_exists(&self, address: &Pubkey) -> ClientResult<I, bool> {
        self.network
            .account_exists(address)
            .await
            .map_err(|e| ClientError::Interface(e))
    }

    pub async fn get_latest_blockhash(&self) -> ClientResult<I, Hash> {
        self.network
            .get_latest_blockhash()
            .await
            .map_err(|e| ClientError::Interface(e))
    }

    pub async fn send(&self, transaction: &impl ToTransaction) -> ClientResult<I, ()> {
        self.send_ordered([transaction]).await
    }

    pub async fn send_ordered(
        &self,
        transactions: impl IntoIterator<Item = impl ToTransaction>,
    ) -> ClientResult<I, ()> {
        let recent_blockhash = self.get_latest_blockhash().await?;
        let txs = transactions
            .into_iter()
            .map(|tx| tx.to_transaction(&self.signer(), recent_blockhash))
            .collect::<Vec<_>>();

        log::debug!("sending {} transactions", txs.len());
        let (signatures, error) = self.network.send_ordered(&txs).await;

        for (index, signature) in signatures.iter().enumerate() {
            log::info!("tx result success: #{index} {signature}");
        }

        let mut tx_log = self.tx_log.lock().unwrap();
        tx_log.extend(&signatures);

        if let Some(error) = error {
            log::error!("tx result failed: #{}: {error:?}", signatures.len());
            return Err(ClientError::Interface(error));
        }

        Ok(())
    }

    pub async fn _send_unordered(
        &self,
        transactions: &[impl ToTransaction],
    ) -> ClientResult<I, Vec<(usize, I::Error)>> {
        let recent_blockhash = self.get_latest_blockhash().await?;
        let txs = transactions
            .iter()
            .map(|tx| tx.to_transaction(&self.signer(), recent_blockhash))
            .collect::<Vec<_>>();

        let results = self
            .network
            .send_unordered(&txs, Some(recent_blockhash))
            .await;

        Ok(results
            .into_iter()
            .enumerate()
            .filter_map(|(i, result)| result.err().map(|e| (i, e)))
            .collect())
    }

    pub(crate) async fn with_wallet_account(
        &self,
        token: &Pubkey,
        ixns: &mut Vec<Instruction>,
    ) -> ClientResult<I, Pubkey> {
        let address = get_associated_token_address(&self.signer(), token);

        if !self.account_exists(&address).await? {
            ixns.push(create_associated_token_account(
                &self.signer(),
                &self.signer(),
                token,
                &spl_token::ID,
            ));
        }

        Ok(address)
    }
}
