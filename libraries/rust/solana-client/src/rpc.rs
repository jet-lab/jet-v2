use std::time::Duration;

use async_trait::async_trait;

use thiserror::Error;

use anchor_lang::{AccountDeserialize, Discriminator, Owner};
use solana_sdk::{
    account::{Account, ReadableAccount},
    hash::Hash,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    signature::Signature,
    transaction::{Transaction, TransactionError, VersionedTransaction},
};
use spl_token::state::Account as TokenAccount;

use solana_transaction_status::TransactionStatus;

#[cfg(feature = "client-native")]
pub mod native;

#[cfg(feature = "client-wasm")]
pub mod wasm;

/// Description of an error occurring while interacting with a Solana RPC node.
#[derive(Error, Debug)]
pub enum ClientError {
    /// The error returned while processing a transaction
    #[error("tx error: {0}")]
    TransactionError(#[from] TransactionError),

    /// The error returned when an expected account is missing
    #[error("account {0} not found")]
    AccountNotFound(Pubkey),

    /// The RPC node returned some invalid value
    #[error("invalid response from rpc: {0}")]
    InvalidResponse(String),

    /// Simple description for some other kind of error
    #[error("solana client error: {0}")]
    Other(String),
}

pub type ClientResult<T> = Result<T, ClientError>;

/// Specify filter requirements when doing an account search
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountFilter {
    Memcmp { offset: usize, bytes: Vec<u8> },
    DataSize(usize),
}

impl AccountFilter {
    pub fn matches(&self, account: &impl ReadableAccount) -> bool {
        match self {
            AccountFilter::Memcmp { offset, bytes } => {
                account.data().len() >= *offset + bytes.len()
                    && account.data()[*offset..*offset + bytes.len()] == *bytes
            }
            AccountFilter::DataSize(size) => account.data().len() == *size,
        }
    }
}

/// A type that allows for interacting with a Solana RPC node
#[async_trait]
pub trait SolanaRpc: Send + Sync {
    async fn get_genesis_hash(&self) -> ClientResult<Hash>;
    async fn get_latest_blockhash(&self) -> ClientResult<Hash>;
    async fn get_slot(&self) -> ClientResult<u64>;
    async fn get_block_time(&self, slot: u64) -> ClientResult<i64>;

    async fn get_multiple_accounts(&self, pubkeys: &[Pubkey])
        -> ClientResult<Vec<Option<Account>>>;

    async fn get_signature_statuses(
        &self,
        signatures: &[Signature],
    ) -> ClientResult<Vec<Option<TransactionStatus>>>;

    async fn airdrop(&self, account: &Pubkey, lamports: u64) -> ClientResult<()>;

    async fn send_transaction_legacy(&self, transaction: &Transaction) -> ClientResult<Signature>;

    async fn send_transaction(&self, transaction: &VersionedTransaction)
        -> ClientResult<Signature>;

    async fn get_program_accounts(
        &self,
        program: &Pubkey,
        filters: &[AccountFilter],
    ) -> ClientResult<Vec<(Pubkey, Account)>>;

    async fn get_token_accounts_by_owner(
        &self,
        owner: &Pubkey,
    ) -> ClientResult<Vec<(Pubkey, TokenAccount)>>;

    async fn get_account(&self, pubkey: &Pubkey) -> ClientResult<Option<Account>> {
        self.get_multiple_accounts(&[*pubkey])
            .await
            .map(|mut accounts| accounts.pop().unwrap())
    }

    async fn get_signature_status(
        &self,
        signature: &Signature,
    ) -> ClientResult<Option<TransactionStatus>> {
        self.get_signature_statuses(&[*signature])
            .await
            .map(|mut statuses| statuses.pop().unwrap())
    }
}

/// Extra helper functions for using the Solana RPC API
#[async_trait]
pub trait SolanaRpcExtra: SolanaRpc {
    /// Confirm the status of submitted transactions
    async fn confirm_transactions(
        &self,
        signatures: &[Signature],
    ) -> Result<Vec<TransactionStatus>, ClientError> {
        for _ in 0..9 {
            let statuses = self.get_signature_statuses(signatures).await?;

            if statuses.iter().all(|s| s.is_some()) {
                return Ok(statuses.into_iter().map(|s| s.unwrap()).collect());
            }

            // come back later
            tokio::time::sleep(Duration::from_millis(250)).await;
        }

        Err(ClientError::Other(format!(
            "did not confirm all transactions: {signatures:?}"
        )))
    }

    /// Confirm a transaction, generate an error result if the transaction failed
    async fn confirm_transaction_result(
        &self,
        signature: Signature,
    ) -> Result<Signature, ClientError> {
        let status = self
            .confirm_transactions(&[signature])
            .await?
            .pop()
            .unwrap();

        match status.err {
            None => Ok(signature),
            Some(err) => Err(ClientError::TransactionError(err)),
        }
    }

    /// Submit a transaction and wait for the result
    async fn send_and_confirm_transaction_legacy(
        &self,
        transaction: &Transaction,
    ) -> Result<Signature, ClientError> {
        let signature = self.send_transaction_legacy(transaction).await?;
        self.confirm_transaction_result(signature).await
    }

    /// Submit a transaction and wait for the result
    async fn send_and_confirm_transaction(
        &self,
        transaction: &VersionedTransaction,
    ) -> Result<Signature, ClientError> {
        let signature = self.send_transaction(transaction).await?;
        self.confirm_transaction_result(signature).await
    }

    /// Check if an account exists (has lamports)
    async fn account_exists(&self, address: &Pubkey) -> ClientResult<bool> {
        Ok(self.get_account(address).await?.is_some())
    }

    /// Retrieve a list of accounts
    ///
    /// This will make multiple RPC requests if the number of accounts is greater than the limit
    /// for a single request.
    async fn get_accounts_all(&self, addresses: &[Pubkey]) -> ClientResult<Vec<Option<Account>>> {
        let mut result = vec![];

        for chunk in addresses.chunks(100) {
            result.extend(self.get_multiple_accounts(chunk).await?);
        }

        Ok(result)
    }

    /// Retrieve states for a list of accounts with a specific type implementing `Pack`
    async fn try_get_packed_accounts<T: Pack + IsInitialized>(
        &self,
        addresses: &[Pubkey],
    ) -> ClientResult<Vec<Option<T>>> {
        let accounts = self.get_accounts_all(addresses).await?;

        accounts
            .into_iter()
            .enumerate()
            .map(|(i, account)| match account {
                Some(a) => T::unpack(a.data())
                    .map_err(|_| {
                        ClientError::Other(format!(
                            "invalid account {}, trying to deserialize type {}",
                            addresses[i],
                            std::any::type_name::<T>()
                        ))
                    })
                    .map(Some),

                None => Ok(None),
            })
            .collect()
    }

    /// Get the state for an account with a known type
    async fn try_get_packed_account<T: Pack + IsInitialized>(
        &self,
        address: &Pubkey,
    ) -> ClientResult<Option<T>> {
        Ok(self
            .try_get_packed_accounts(&[*address])
            .await?
            .pop()
            .unwrap())
    }

    /// Get the state for an account with a known type
    async fn get_packed_account<T: Pack + IsInitialized>(
        &self,
        address: &Pubkey,
    ) -> ClientResult<T> {
        self.try_get_packed_account(address)
            .await?
            .ok_or(ClientError::AccountNotFound(*address))
    }

    /// Get the state for a token account
    async fn get_token_account(&self, address: &Pubkey) -> ClientResult<TokenAccount> {
        self.get_packed_account(address).await
    }

    /// Retrieve states for a list of accounts with a specific type implementing `AccountDeserialize` from anchor
    async fn try_get_anchor_accounts<T: AccountDeserialize>(
        &self,
        addresses: &[Pubkey],
    ) -> ClientResult<Vec<Option<T>>> {
        let accounts = self.get_accounts_all(addresses).await?;

        accounts
            .into_iter()
            .enumerate()
            .map(|(i, account)| match account {
                Some(a) => T::try_deserialize(&mut a.data())
                    .map_err(|_| {
                        ClientError::Other(format!(
                            "invalid account {}, trying to deserialize type {}",
                            addresses[i],
                            std::any::type_name::<T>()
                        ))
                    })
                    .map(Some),

                None => Ok(None),
            })
            .collect()
    }

    /// Get the state for an account with a known type
    async fn try_get_anchor_account<T: AccountDeserialize>(
        &self,
        address: &Pubkey,
    ) -> ClientResult<Option<T>> {
        Ok(self
            .try_get_anchor_accounts(&[*address])
            .await?
            .pop()
            .unwrap())
    }

    /// Get the state for an account with a known type
    async fn get_anchor_account<T: AccountDeserialize>(&self, address: &Pubkey) -> ClientResult<T> {
        self.try_get_anchor_account(address)
            .await?
            .ok_or(ClientError::AccountNotFound(*address))
    }

    /// Retrieve a list of accounts by their serializable anchor type
    async fn find_anchor_accounts<T: AccountDeserialize + Owner + Discriminator>(
        &self,
    ) -> ClientResult<Vec<(Pubkey, T)>> {
        let program = T::owner();

        let accounts = self
            .get_program_accounts(
                &program,
                &[AccountFilter::Memcmp {
                    offset: 0,
                    bytes: T::discriminator().to_vec(),
                }],
            )
            .await?;

        Ok(accounts
            .into_iter()
            .filter_map(
                |(pubkey, account)| match T::try_deserialize(&mut account.data()) {
                    Ok(state) => Some((pubkey, state)),
                    Err(_) => {
                        log::warn!(
                            "invalid account {}, trying to deserialize type {}",
                            pubkey,
                            std::any::type_name::<T>()
                        );

                        None
                    }
                },
            )
            .collect())
    }
}

impl<Rpc: SolanaRpc> SolanaRpcExtra for Rpc {}
impl SolanaRpcExtra for dyn SolanaRpc {}
