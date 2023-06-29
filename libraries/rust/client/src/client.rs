use std::{
    collections::VecDeque,
    error::Error as StdError,
    rc::Rc,
    sync::{Arc, Mutex},
};
use thiserror::Error;

use solana_sdk::{
    address_lookup_table_account::AddressLookupTableAccount, hash::Hash, instruction::Instruction,
    pubkey::Pubkey, signature::Signature,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};

use jet_solana_client::{
    rpc::{SolanaRpc, SolanaRpcExtra},
    transaction::{compile_versioned_transaction, ToTransaction},
};

use crate::{config::JetAppConfig, state::AccountStates, Wallet};

pub type ClientResult<T> = std::result::Result<T, ClientError>;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("rpc client error")]
    Rpc(#[from] jet_solana_client::rpc::ClientError),
    #[error("decode error: {0}")]
    Deserialize(Box<dyn StdError + Send + Sync>),
    #[error("wallet is not connected")]
    MissingWallet,
    #[error("error: {0}")]
    Unexpected(String),
}

impl From<bincode::Error> for ClientError {
    fn from(err: bincode::Error) -> Self {
        Self::Unexpected(format!("Unexpected encoding error: {err:?}"))
    }
}

impl From<anchor_lang::error::Error> for ClientError {
    fn from(err: anchor_lang::error::Error) -> Self {
        Self::Unexpected(format!("Unexpected Anchor error: {err:?}"))
    }
}

impl From<solana_sdk::instruction::InstructionError> for ClientError {
    fn from(err: solana_sdk::instruction::InstructionError) -> Self {
        Self::Unexpected(format!("Unexpected Solana instruction error: {err:?}"))
    }
}

/// Central object for client implementations, containing the global configuration and any
/// caching for account data.
pub struct ClientState {
    pub(crate) network: Arc<dyn SolanaRpc>,
    pub(crate) pubkey: Pubkey,
    wallet: Rc<dyn Wallet>,
    state: AccountStates,
    tx_log: Mutex<VecDeque<Signature>>,
}

impl ClientState {
    pub fn new(
        network: Arc<dyn SolanaRpc>,
        wallet: Rc<dyn Wallet>,
        config: JetAppConfig,
        airspace: String,
    ) -> ClientResult<Self> {
        let Some(pubkey) = wallet.pubkey() else {
            return Err(ClientError::MissingWallet);
        };

        Ok(Self {
            state: AccountStates::new(network.clone(), pubkey, config, airspace)?,
            tx_log: Mutex::new(VecDeque::new()),
            network,
            wallet,
            pubkey,
        })
    }

    pub fn signer(&self) -> Pubkey {
        self.pubkey
    }

    pub fn airspace(&self) -> Pubkey {
        self.state.config.airspace
    }

    pub fn airspace_lookup_registry_authority(&self) -> Option<Pubkey> {
        self.state.config.airspace_lookup_registry_authority
    }

    pub fn state(&self) -> &AccountStates {
        &self.state
    }

    pub async fn account_exists(&self, address: &Pubkey) -> ClientResult<bool> {
        Ok(self.network.account_exists(address).await?)
    }

    pub async fn get_latest_blockhash(&self) -> ClientResult<Hash> {
        Ok(self.network.get_latest_blockhash().await?)
    }

    pub async fn send(&self, transaction: &impl ToTransaction) -> ClientResult<()> {
        self.send_ordered([transaction]).await
    }

    pub async fn get_slot(&self) -> ClientResult<u64> {
        Ok(self.network.get_slot().await?)
    }

    pub async fn send_ordered(
        &self,
        transactions: impl IntoIterator<Item = impl ToTransaction>,
    ) -> ClientResult<()> {
        let tx_to_send = transactions.into_iter().collect::<Vec<_>>();
        let mut signatures = vec![];
        let mut error = None;

        log::debug!("sending {} transactions", tx_to_send.len());
        for (index, tx) in tx_to_send.into_iter().enumerate() {
            let recent_blockhash = self.get_latest_blockhash().await?;
            let tx = tx.to_transaction(&self.signer(), recent_blockhash);
            let tx = self
                .wallet
                .sign_transactions(&[tx])
                .await
                .ok_or(ClientError::MissingWallet)?
                .pop()
                .unwrap();

            let signature = match self.network.send_transaction(&tx).await {
                Err(err) => {
                    log::error!("failed sending transaction: #{index}: {err:?}");
                    error = Some(err);
                    break;
                }

                Ok(signature) => {
                    log::info!("submitted transaction #{index}: {signature}");
                    signatures.push(signature);

                    signature
                }
            };

            self.network.confirm_transaction_result(signature).await?;
        }

        let mut tx_log = self.tx_log.lock().unwrap();
        tx_log.extend(&signatures);

        match error {
            Some(e) => Err(e.into()),
            None => Ok(()),
        }
    }

    pub async fn send_with_lookup_tables(
        &self,
        instructions: &[Instruction],
        lookup_tables: &[AddressLookupTableAccount],
    ) -> ClientResult<()> {
        let recent_blockhash = self.get_latest_blockhash().await?;
        let tx = compile_versioned_transaction(
            instructions,
            &self.signer(),
            lookup_tables,
            recent_blockhash,
        )
        .map_err(|e| ClientError::Unexpected(format!("compile error: {e:?}")))?;

        let tx = &self.wallet.sign_transactions(&[tx]).await.unwrap()[0];
        let signature = self.network.send_transaction(tx).await?;

        self.network.confirm_transaction_result(signature).await?;

        log::info!("tx result success: {signature}");

        let mut tx_log = self.tx_log.lock().unwrap();
        tx_log.push_back(signature);

        Ok(())
    }

    pub(crate) async fn with_wallet_account(
        &self,
        token: &Pubkey,
        ixns: &mut Vec<Instruction>,
    ) -> ClientResult<Pubkey> {
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
