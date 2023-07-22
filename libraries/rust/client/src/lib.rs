use std::{rc::Rc, sync::Arc};

use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;

use jet_solana_client::rpc::SolanaRpc;

use client::ClientState;
use config::JetAppConfig;
use fixed_term::FixedTermMarketClient;
use margin::MarginClient;
use state::{tokens::TokenAccount, AccountStates};
use test_service::TestServiceClient;

mod client;
pub mod config;
pub mod fixed_term;
pub mod margin;
pub mod margin_orca;
pub mod margin_pool;
pub mod state;
pub mod swaps;
pub mod test_service;
mod wallet;

pub use client::{ClientError, ClientResult};
pub use jet_solana_client::network::NetworkKind;
pub use wallet::Wallet;

/// Central client object for interacting with the protocol
#[derive(Clone)]
pub struct JetClient {
    client: Arc<ClientState>,
}

impl JetClient {
    /// Create the client state
    pub fn new(
        interface: Arc<dyn SolanaRpc>,
        wallet: Rc<dyn Wallet>,
        config: JetAppConfig,
        airspace: &str,
    ) -> ClientResult<Self> {
        Ok(Self {
            client: Arc::new(ClientState::new(
                interface,
                wallet,
                config,
                airspace.to_owned(),
            )?),
        })
    }

    /// The airspace this client is associated with
    pub fn airspace(&self) -> Pubkey {
        self.client.airspace()
    }

    /// Get the state management object for this client
    pub fn state(&self) -> &AccountStates {
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
    pub fn test_service(&self) -> TestServiceClient {
        TestServiceClient::new(self.client.clone())
    }

    /// Get the client for the margin program
    pub fn margin(&self) -> MarginClient {
        MarginClient::new(self.client.clone())
    }

    /// Get the client for the fixed term markets program
    pub fn fixed_term(&self) -> FixedTermMarketClient {
        FixedTermMarketClient::new(self.client.clone())
    }
}

macro_rules! bail {
    ($($fmt_args:tt)*) => {
        return Err($crate::client::ClientError::Unexpected(format!($($fmt_args)*)))
    };
}

pub(crate) use bail;
