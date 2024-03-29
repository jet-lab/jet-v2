use std::{rc::Rc, sync::Arc};

use jet_solana_client::rpc::wasm::RpcConnection;
use serde_json::Value;
use wasm_bindgen::{prelude::*, JsCast};

use solana_sdk::pubkey::Pubkey;

use jet_client::{
    config::{CONFIG_URL_DEVNET, CONFIG_URL_MAINNET},
    state::tokens::TokenAccount,
    test_service::TestServiceClient,
    JetClient, NetworkKind,
};

/// Bindings for the @soalana/web3.js library
mod solana_web3;

mod error;
mod wallet;

pub mod fixed_term;
pub mod margin;
pub mod margin_pool;

pub use error::ClientError;
use wallet::WalletAdapter;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[wasm_bindgen]
pub struct JetWebClient {
    client: JetClient,
}

#[wasm_bindgen]
impl JetWebClient {
    pub async fn connect(
        wallet: WalletAdapter,
        url: &str,
        airspace_name: &str,
    ) -> Result<JetWebClient, ClientError> {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        let rpc = Arc::new(RpcConnection::new(url));
        let network_kind = NetworkKind::from_interface(rpc.as_ref()).await?;

        let config_url = match network_kind {
            NetworkKind::Mainnet => CONFIG_URL_MAINNET,
            NetworkKind::Devnet => CONFIG_URL_DEVNET,
            NetworkKind::Localnet => "/localnet.config.json",
        };
        let config_request = {
            let mut opts = RequestInit::new();
            opts.method("GET");
            opts.mode(RequestMode::Cors);

            Request::new_with_str_and_init(config_url, &opts).unwrap()
        };

        let log_level = match network_kind {
            NetworkKind::Localnet | NetworkKind::Devnet => log::Level::Debug,
            NetworkKind::Mainnet => log::Level::Warn,
        };

        if console_log::init_with_level(log_level).is_err() {
            console_log!("Unable to initialize console log, might already be initialized");
        }

        let config_response = {
            let window = web_sys::window().unwrap();
            let resp_value = JsFuture::from(window.fetch_with_request(&config_request))
                .await
                .unwrap();

            // `resp_value` is a `Response` object.
            assert!(resp_value.is_instance_of::<Response>());
            let resp: Response = resp_value.dyn_into().unwrap();

            // Convert this other `Promise` into a rust `Future`.
            let json = JsFuture::from(resp.json().unwrap()).await.unwrap();
            serde_wasm_bindgen::from_value::<Value>(json).unwrap()
        };

        let config = serde_json::from_value(config_response).unwrap();

        let wallet = Rc::new(wallet);

        Ok(Self {
            client: JetClient::new(rpc, wallet, config, airspace_name)?,
        })
    }

    pub fn state(&self) -> ClientState {
        ClientState {
            inner: self.client.clone(),
        }
    }

    /// Client object for interacting with the test-service program available in test environments
    #[wasm_bindgen(js_name = testService)]
    pub fn test_service(&self) -> TestServiceWebClient {
        TestServiceWebClient {
            inner: self.client.test_service(),
        }
    }

    pub fn margin(&self) -> margin::MarginWebClient {
        margin::MarginWebClient(self.client.margin())
    }
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct ClientState {
    inner: JetClient,
}

#[wasm_bindgen]
impl ClientState {
    #[wasm_bindgen(js_name = walletBalance)]
    pub fn wallet_balance(&self, token: &Pubkey) -> u64 {
        self.inner
            .state()
            .get::<TokenAccount>(token)
            .map(|a| a.amount)
            .unwrap_or_default()
    }

    #[wasm_bindgen(js_name = syncAll)]
    pub async fn sync_all(&self) -> Result<(), ClientError> {
        self.sync_oracles().await?;

        Ok(())
    }

    #[wasm_bindgen(js_name = syncAccounts)]
    pub async fn sync_accounts(&self) -> Result<(), ClientError> {
        jet_client::state::margin::sync_margin_accounts(self.inner.state()).await?;
        jet_client::state::fixed_term::sync_user_accounts(self.inner.state()).await?;

        Ok(())
    }

    #[wasm_bindgen(js_name = syncOracles)]
    pub async fn sync_oracles(&self) -> Result<(), ClientError> {
        jet_client::state::oracles::sync(self.inner.state()).await?;

        Ok(())
    }
}

#[wasm_bindgen]
pub struct TestServiceWebClient {
    inner: TestServiceClient,
}

#[wasm_bindgen]
impl TestServiceWebClient {
    /// Request some amount of tokens for the current user
    #[wasm_bindgen(js_name = tokenRequest)]
    pub async fn token_request(&self, mint: &Pubkey, amount: u64) -> Result<(), ClientError> {
        Ok(self.inner.token_request(mint, amount).await?)
    }
}

#[wasm_bindgen(start, js_name = initModule)]
pub fn init_module() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => ($crate::log(&format_args!($($t)*).to_string()))
}
