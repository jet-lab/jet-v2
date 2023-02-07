use std::any::Any;

use anchor_lang::AccountDeserialize;
use async_trait::async_trait;
use thiserror::Error;

use solana_sdk::{
    account::Account, hash::Hash, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
    signature::Signature, transaction::VersionedTransaction,
};
use transaction::{condense, ToTransaction, TransactionBuilder};

pub mod network;
pub mod transaction;
mod util;

/// A type that provides an interface to interact with the Solana network, and an associated
/// wallet that can sign transactions to be sent to the network.
#[async_trait(?Send)]
pub trait NetworkUserInterface: Clone + 'static {
    type Error: Any + std::fmt::Debug;

    /// The signing address used by this interface when sending transactions
    fn signer(&self) -> Pubkey;

    /// The current time
    fn get_current_time(&self) -> i64;

    /// Get the genesis hash for the networh
    async fn get_genesis_hash(&self) -> Result<Hash, Self::Error>;

    /// Get the latest blockhash from the network
    async fn get_latest_blockhash(&self) -> Result<Hash, Self::Error>;

    /// Retrieve multiple accounts in one operation
    async fn get_accounts(&self, addresses: &[Pubkey])
        -> Result<Vec<Option<Account>>, Self::Error>;

    /// Send a set of transactions to the network
    ///
    /// Must assume the transactions should be submitted in-order
    async fn send_ordered(
        &self,
        transactions: &[VersionedTransaction],
    ) -> (Vec<Signature>, Option<Self::Error>);

    /// Send a set of transactions to the network
    ///
    /// Can assmume that the order of the provided transactions does not matter,
    /// which may allow them to be executed faster concurrently.
    async fn send_unordered(
        &self,
        transactions: &[VersionedTransaction],
        blockhash: Option<Hash>,
    ) -> Vec<Result<Signature, Self::Error>>;

    /// Send a transaction message to the network
    async fn send(&self, transaction: VersionedTransaction) -> Result<Signature, Self::Error> {
        let (mut signatures, error) = self.send_ordered(&[transaction]).await;

        match signatures.pop() {
            Some(signature) => Ok(signature),
            None => Err(error.unwrap()),
        }
    }

    /// Check if accounts exist (is funded)
    async fn accounts_exist(&self, addresses: &[Pubkey]) -> Result<Vec<bool>, Self::Error> {
        Ok(self
            .get_accounts(addresses)
            .await?
            .into_iter()
            .map(|maybe_acc| maybe_acc.is_some())
            .collect())
    }

    /// Check if an account exists (is funded)
    async fn account_exists(&self, address: &Pubkey) -> Result<bool, Self::Error> {
        Ok(self.accounts_exist(&[*address]).await?[0])
    }
}

#[async_trait(?Send)]
pub trait NetworkUserInterfaceExt: NetworkUserInterface {
    async fn get_accounts_all(
        &self,
        addresses: &[Pubkey],
    ) -> Result<Vec<Option<Account>>, ExtError<Self>> {
        let mut result = vec![];

        for chunk in addresses.chunks(100) {
            let accounts = self
                .get_accounts(chunk)
                .await
                .map_err(|e| ExtError::Interface(e))?;

            result.extend(accounts);
        }

        Ok(result)
    }

    async fn get_account(&self, address: &Pubkey) -> Result<Option<Account>, ExtError<Self>> {
        self.get_accounts_all(&[*address])
            .await
            .map(|list| list.into_iter().next().unwrap())
    }

    async fn account_exists(&self, address: &Pubkey) -> Result<bool, ExtError<Self>> {
        self.get_account(address)
            .await
            .map(|account| account.is_some())
    }

    async fn get_token_account(
        &self,
        address: &Pubkey,
    ) -> Result<Option<spl_token::state::Account>, ExtError<Self>> {
        match self.get_account(address).await? {
            None => Ok(None),
            Some(account) => spl_token::state::Account::unpack(&account.data)
                .map(Some)
                .map_err(|e| ExtError::Unpack {
                    address: *address,
                    error: e,
                }),
        }
    }

    async fn get_mint(
        &self,
        address: &Pubkey,
    ) -> Result<Option<spl_token::state::Mint>, ExtError<Self>> {
        match self.get_account(address).await? {
            None => Ok(None),
            Some(account) => spl_token::state::Mint::unpack(&account.data)
                .map(Some)
                .map_err(|e| ExtError::Unpack {
                    address: *address,
                    error: e,
                }),
        }
    }

    async fn get_anchor_accounts<T: AccountDeserialize>(
        &self,
        addresses: &[Pubkey],
    ) -> Result<Vec<Option<T>>, ExtError<Self>> {
        self.get_accounts_all(addresses)
            .await?
            .into_iter()
            .enumerate()
            .map(|(i, account_info)| match account_info {
                None => Ok(None),
                Some(account) => T::try_deserialize(&mut &account.data[..])
                    .map(|a| Some(a))
                    .map_err(|e| ExtError::Deserialize {
                        address: addresses[i],
                        error: e,
                    }),
            })
            .collect()
    }

    async fn get_anchor_account<T: AccountDeserialize>(
        &self,
        address: &Pubkey,
    ) -> Result<Option<T>, ExtError<Self>> {
        Ok(self.get_anchor_accounts(&[*address]).await?.pop().unwrap())
    }

    async fn send_condensed_ordered(
        &self,
        txns: &[TransactionBuilder],
    ) -> (Vec<Signature>, Option<ExtError<Self>>) {
        let txns = condense(txns).unwrap();
        let blockhash = match self.get_latest_blockhash().await {
            Ok(hash) => hash,
            Err(e) => return (vec![], Some(ExtError::Interface(e))),
        };

        let txns = txns
            .into_iter()
            .map(|tx| tx.to_transaction(&self.signer(), blockhash))
            .collect::<Vec<_>>();

        let (sigs, error) = self.send_ordered(&txns).await;

        (sigs, error.map(|e| ExtError::Interface(e)))
    }

    async fn send_condensed_unordered(
        &self,
        txns: &[TransactionBuilder],
    ) -> Vec<Result<Signature, ExtError<Self>>> {
        let txns = condense(txns).unwrap();
        let blockhash = match self.get_latest_blockhash().await {
            Ok(hash) => hash,
            Err(e) => return vec![Err(ExtError::Interface(e))],
        };

        let txns = txns
            .into_iter()
            .map(|tx| tx.to_transaction(&self.signer(), blockhash))
            .collect::<Vec<_>>();

        self.send_unordered(&txns, Some(blockhash))
            .await
            .into_iter()
            .map(|result| result.map_err(|e| ExtError::Interface(e)))
            .collect()
    }
}

#[derive(Error, Debug)]
pub enum ExtError<I: NetworkUserInterface> {
    #[error("interface error")]
    Interface(I::Error),

    #[error("error unpacking account {address}: {error}")]
    Unpack {
        address: Pubkey,
        error: ProgramError,
    },

    #[error("error deserializing account {address}: {error}")]
    Deserialize {
        address: Pubkey,
        error: anchor_lang::error::Error,
    },
}

impl<T: NetworkUserInterface> NetworkUserInterfaceExt for T {}
