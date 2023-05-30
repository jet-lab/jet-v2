use std::str::FromStr;

use async_trait::async_trait;
use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::*;

use solana_sdk::pubkey::Pubkey;

use jet_client::Wallet;

#[wasm_bindgen]
extern "C" {
    /// A JS type that can interact with a browser wallet
    #[derive(Clone)]
    pub type WalletAdapter;

    /// Get the address of the connected wallet
    #[wasm_bindgen(method, catch, js_name = connected)]
    pub fn connected(this: &WalletAdapter) -> Result<Option<String>, js_sys::Error>;

    /// Request the wallet so sign arbitrary messages
    #[wasm_bindgen(method, catch, js_name = signMessages)]
    pub fn sign_messages(
        this: &WalletAdapter,
        messages: Vec<Uint8Array>,
    ) -> Result<JsValue, js_sys::Error>;

    /// Request the wallet to sign a list of transactions
    #[wasm_bindgen(method, catch, js_name = signTransactions)]
    pub async fn sign_transactions(
        this: &WalletAdapter,
        transactions: Vec<Uint8Array>,
    ) -> Result<JsValue, js_sys::Error>;
}

#[async_trait(?Send)]
impl Wallet for WalletAdapter {
    fn pubkey(&self) -> Option<Pubkey> {
        self.connected()
            .unwrap()
            .map(|s| Pubkey::from_str(&s).unwrap())
    }

    async fn sign_messages(
        &self,
        messages: &[&[u8]],
    ) -> Option<Vec<solana_sdk::signature::Signature>> {
        let messages = messages
            .iter()
            .map(|msg| Uint8Array::from(*msg))
            .collect::<Vec<_>>();

        let signatures = self.sign_messages(messages).ok()?;

        Some(serde_wasm_bindgen::from_value(signatures).unwrap())
    }

    async fn sign_transactions(
        &self,
        transactions: &[solana_sdk::transaction::VersionedTransaction],
    ) -> Option<Vec<solana_sdk::transaction::VersionedTransaction>> {
        let transactions = transactions
            .iter()
            .map(|tx| Uint8Array::from(bincode::serialize(&tx).unwrap().as_ref()))
            .collect::<Vec<_>>();

        let transactions = self.sign_transactions(transactions).await.ok()?;

        Some(
            Array::from(&transactions)
                .iter()
                .map(|tx| {
                    let tx = Uint8Array::from(tx).to_vec();
                    bincode::deserialize(&tx).unwrap()
                })
                .collect::<Vec<_>>(),
        )
    }
}
