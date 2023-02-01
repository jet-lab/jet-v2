use std::str::FromStr;

use serde_json::Value;
use wasm_bindgen::{prelude::*, JsCast};

use solana_sdk::{hash::Hash, pubkey::Pubkey};

use jet_client::{
    config::{AirspaceInfo, JetAppConfig, JetAppConfigOld, TokenInfo},
    state::tokens::TokenAccount,
    test_service::TestServiceClient,
    JetClient, NetworkKind,
};

/// Bindings for the @soalana/web3.js library
mod solana_web3;

mod network_adapter;

pub mod fixed_term;
pub mod margin;
pub mod margin_pool;

use network_adapter::{JsNetworkAdapter, SolanaNetworkAdapter};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[wasm_bindgen]
pub struct JetWebClient {
    client: JetClient<JsNetworkAdapter>,
}

#[wasm_bindgen]
impl JetWebClient {
    pub async fn connect(
        user_address: Pubkey,
        adapter: SolanaNetworkAdapter,
        legacy_config: bool,
    ) -> Result<JetWebClient, JsError> {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        let network_genesis_hash = adapter
            .get_genesis_hash()
            .await
            .unwrap()
            .as_string()
            .and_then(|str| Hash::from_str(&str).ok())
            .ok_or_else(|| js_sys::Error::new("invalid network genesis hash"))
            .unwrap();

        let network_kind = NetworkKind::from_genesis_hash(&network_genesis_hash);
        let config_url = match network_kind {
            NetworkKind::Mainnet | NetworkKind::Devnet => JetAppConfig::DEFAULT_URL,
            NetworkKind::Localnet => "/localnet.config.json",
        };
        let config_request = {
            let mut opts = RequestInit::new();
            opts.method("GET");
            opts.mode(RequestMode::Cors);

            Request::new_with_str_and_init(config_url, &opts).unwrap()
        };

        let log_level = if network_kind == NetworkKind::Localnet {
            log::Level::Debug
        } else {
            log::Level::Warn
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

        let config = if legacy_config {
            let legacy_config: JetAppConfigOld = serde_json::from_value(config_response).unwrap();

            let tokens_as_vec: Vec<TokenInfo> = legacy_config.tokens.values().cloned().collect();
            let airspaces = legacy_config
                .airspaces
                .into_iter()
                .map(|airspace| AirspaceInfo {
                    name: airspace.name,
                    tokens: airspace.tokens,
                    fixed_term_markets: airspace
                        .fixed_term_markets
                        .values()
                        .into_iter()
                        .map(|market| Pubkey::from_str(&market.market).unwrap())
                        .collect(),
                })
                .collect();

            JetAppConfig {
                tokens: tokens_as_vec,
                airspaces,
                exchanges: vec![],
            }
        } else {
            serde_json::from_value(config_response).unwrap()
        };

        let adapter = JsNetworkAdapter::new(adapter, user_address);

        Ok(Self {
            client: JetClient::new(adapter, config, "default")?,
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
pub struct ClientError {
    value: js_sys::Error,
}

impl From<ClientError> for JsValue {
    fn from(this: ClientError) -> Self {
        this.value.into()
    }
}

impl From<jet_client::ClientError<JsNetworkAdapter>> for ClientError {
    fn from(err: jet_client::ClientError<JsNetworkAdapter>) -> Self {
        match err {
            jet_client::ClientError::Interface(error) => {
                web_sys::console::log_1(&error);
                Self { value: error }
            }
            rust_err => Self {
                value: js_sys::Error::new(&format!("sdk error: {}", rust_err)),
            },
        }
    }
}

#[wasm_bindgen]
pub struct ClientState {
    inner: JetClient<JsNetworkAdapter>,
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
    inner: TestServiceClient<JsNetworkAdapter>,
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
