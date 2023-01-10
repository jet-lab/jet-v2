use std::{str::FromStr, sync::Arc};

use anchor_lang::AccountDeserialize;
use async_trait::async_trait;
use client::ClientState;
use config::JetAppConfig;
use fixed_term::FixedTermMarketClient;
use margin::MarginClient;
use solana_sdk::{
    account::Account, hash::Hash, program_pack::Pack, pubkey::Pubkey, signature::Signature,
    transaction::VersionedTransaction,
};
use spl_associated_token_account::get_associated_token_address;
use state::{tokens::TokenAccount, AccountStates};
use test_service::TestServiceClient;

mod client;
pub mod config;
pub mod fixed_term;
pub mod margin;
pub mod margin_pool;
pub mod programs;
mod solana;
pub mod state;
pub mod swaps;
pub mod test_service;
pub mod util;

pub use client::{ClientError, ClientResult};

const MAINNET_HASH: &str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d";
const DEVNET_HASH: &str = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG";

/// A type that provides an interface to interact with the Solana network, and an associated
/// wallet that can sign transactions to be sent to the network.
#[async_trait(?Send)]
pub trait UserNetworkInterface: Clone {
    type Error: std::any::Any + std::fmt::Debug;

    /// The signing address used by this interface when sending transactions
    fn signer(&self) -> Pubkey;

    /// The current time
    fn get_current_time(&self) -> i64;

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
    ) -> Vec<Result<Signature, Self::Error>>;

    /// Send a transaction message to the network
    async fn send(&self, transaction: VersionedTransaction) -> Result<Signature, Self::Error> {
        let (mut signatures, error) = self.send_ordered(&[transaction]).await;

        match signatures.pop() {
            Some(signature) => Ok(signature),
            None => Err(error.unwrap()),
        }
    }
}

#[async_trait(?Send)]
pub(crate) trait ClientInterfaceExt: UserNetworkInterface {
    async fn get_accounts_all(
        &self,
        addresses: &[Pubkey],
    ) -> ClientResult<Self, Vec<Option<Account>>> {
        let mut result = vec![];

        for chunk in addresses.chunks(100) {
            let accounts = self
                .get_accounts(chunk)
                .await
                .map_err(|e| ClientError::Interface(e))?;

            result.extend(accounts);
        }

        Ok(result)
    }

    async fn get_account(&self, address: &Pubkey) -> ClientResult<Self, Option<Account>> {
        self.get_accounts_all(&[*address])
            .await
            .map(|list| list.into_iter().next().unwrap())
    }

    async fn account_exists(&self, address: &Pubkey) -> ClientResult<Self, bool> {
        self.get_account(address)
            .await
            .map(|account| account.is_some())
    }

    async fn get_token_account(
        &self,
        address: &Pubkey,
    ) -> ClientResult<Self, Option<spl_token::state::Account>> {
        match self.get_account(address).await? {
            None => Ok(None),
            Some(account) => spl_token::state::Account::unpack(&account.data)
                .map(Some)
                .map_err(|e| ClientError::Deserialize(Box::new(e))),
        }
    }

    async fn get_anchor_accounts<T: AccountDeserialize>(
        &self,
        addresses: &[Pubkey],
    ) -> ClientResult<Self, Vec<Option<T>>> {
        self.get_accounts_all(addresses)
            .await?
            .into_iter()
            .map(|account_info| match account_info {
                None => Ok(None),
                Some(account) => T::try_deserialize(&mut &account.data[..])
                    .map(|a| Some(a))
                    .map_err(|e| ClientError::Deserialize(Box::new(e))),
            })
            .collect()
    }

    async fn get_anchor_account<T: AccountDeserialize>(
        &self,
        address: &Pubkey,
    ) -> ClientResult<Self, Option<T>> {
        match self.get_account(address).await? {
            None => Ok(None),
            Some(account) => T::try_deserialize(&mut &account.data[..])
                .map(|a| Some(a))
                .map_err(|e| ClientError::Deserialize(Box::new(e))),
        }
    }
}

impl<T: UserNetworkInterface> ClientInterfaceExt for T {}

/// Central client object for interacting with the protocol
#[derive(Clone)]
pub struct JetClient<I> {
    client: Arc<ClientState<I>>,
}

impl<I: UserNetworkInterface> JetClient<I> {
    /// Create the client state
    pub fn new(interface: I, config: JetAppConfig, airspace: &str) -> ClientResult<I, Self> {
        Ok(Self {
            client: Arc::new(ClientState::new(interface, config, airspace.to_owned())?),
        })
    }

    /// The airspace this client is associated with
    pub fn airspace(&self) -> Pubkey {
        self.client.airspace()
    }

    /// Get the state management object for this client
    pub fn state(&self) -> &AccountStates<I> {
        self.client.state()
    }

    /// Get the balance of a token owned by the user's wallet
    pub fn wallet_balance(&self, token: &Pubkey) -> u64 {
        let address = get_associated_token_address(&self.client.signer(), token);

        self.client
            .state()
            .get::<TokenAccount>(&address)
            .map(|account| account.amount)
            .unwrap_or_default()
    }

    /// Get the client for the test service program
    pub fn test_service(&self) -> TestServiceClient<I> {
        TestServiceClient::new(self.client.clone())
    }

    /// Get the client for the margin program
    pub fn margin(&self) -> MarginClient<I> {
        MarginClient::new(self.client.clone())
    }

    /// Get the client for the fixed term markets program
    pub fn fixed_term(&self) -> FixedTermMarketClient<I> {
        FixedTermMarketClient::new(self.client.clone())
    }
}

/// Description for the Solana network a client may connect to
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum NetworkKind {
    /// The public mainnet-beta network
    Mainnet,

    /// The public network for development testing
    Devnet,

    /// A non-public network for testing
    Localnet,
}

impl NetworkKind {
    /// Determine the network type based on its genesis hash
    pub fn from_genesis_hash(network_genesis_hash: &Hash) -> Self {
        if *network_genesis_hash == Hash::from_str(MAINNET_HASH).unwrap() {
            NetworkKind::Mainnet
        } else if *network_genesis_hash == Hash::from_str(DEVNET_HASH).unwrap() {
            NetworkKind::Devnet
        } else {
            NetworkKind::Localnet
        }
    }
}

macro_rules! bail {
    ($($fmt_args:tt)*) => {
        return Err($crate::client::ClientError::Unexpected(format!($($fmt_args)*)))
    };
}

pub(crate) use bail;
