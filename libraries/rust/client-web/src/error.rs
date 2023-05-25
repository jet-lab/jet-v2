use thiserror::Error;
use wasm_bindgen::JsValue;

/// A wrapper for client errors to work in a JS environment
#[derive(Error, Debug)]
pub enum ClientError {
    /// An error from the rust-side client
    #[error("{0}")]
    Client(#[from] jet_client::ClientError),

    /// An error from the rust-side RPC client
    #[error("{0}")]
    Rpc(#[from] jet_solana_client::rpc::ClientError),

    /// An error from the JS-side
    #[error("{}", .0.message())]
    Js(js_sys::Error),
}

impl From<ClientError> for JsValue {
    fn from(value: ClientError) -> Self {
        match value {
            ClientError::Client(err) => {
                js_sys::Error::new(&format!("client error: {}", err)).into()
            }
            ClientError::Rpc(err) => js_sys::Error::new(&format!("rpc error: {}", err)).into(),
            ClientError::Js(err) => err.into(),
        }
    }
}
