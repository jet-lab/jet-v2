use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "@solana/web3.js")]
extern "C" {
    pub type PublicKey;

    #[wasm_bindgen(constructor)]
    pub fn new(bytes: &[u8]) -> PublicKey;

    #[wasm_bindgen(method, js_name = toBytes)]
    pub fn to_bytes(this: &PublicKey) -> Vec<u8>;

}

#[wasm_bindgen(module = "@solana/web3.js")]
extern "C" {
    pub type Connection;

    #[wasm_bindgen(constructor)]
    pub fn new(endpoint: &str, commitment: &str) -> Connection;

    #[wasm_bindgen(method, catch, js_name = getMultipleAccountsInfo)]
    pub async fn get_accounts(this: &Connection) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch, js_name = getLatestBlockhash)]
    pub async fn get_latest_blockhash(this: &Connection) -> Result<JsValue, JsValue>;
}
