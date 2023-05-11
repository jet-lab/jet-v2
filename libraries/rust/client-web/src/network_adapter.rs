use std::str::FromStr;

use async_trait::async_trait;
use js_sys::{BigInt, Reflect, Uint8Array};
use wasm_bindgen::{prelude::*, JsCast};

use solana_sdk::{
    account::Account, hash::Hash, pubkey::Pubkey, signature::Signature,
    transaction::VersionedTransaction,
};

use jet_solana_client::NetworkUserInterface;

use super::solana_web3;

#[wasm_bindgen]
extern "C" {
    #[derive(Clone)]
    pub type SolanaNetworkAdapter;

    #[wasm_bindgen(method, catch, js_name = getGenesisHash)]
    pub async fn get_genesis_hash(this: &SolanaNetworkAdapter) -> Result<JsValue, js_sys::Error>;

    #[wasm_bindgen(method, catch, js_name = getAccounts)]
    pub async fn get_accounts(
        this: &SolanaNetworkAdapter,
        addresses: Vec<solana_web3::PublicKey>,
    ) -> Result<JsValue, js_sys::Error>;

    #[wasm_bindgen(method, catch, js_name = getLatestBlockhash)]
    pub async fn get_latest_blockhash(
        this: &SolanaNetworkAdapter,
    ) -> Result<JsValue, js_sys::Error>;

    #[wasm_bindgen(method, catch)]
    pub async fn send(
        this: &SolanaNetworkAdapter,
        transactions: JsValue,
    ) -> Result<JsValue, js_sys::Error>;
}

#[derive(Clone)]
pub struct JsNetworkAdapter {
    js_obj: SolanaNetworkAdapter,
    signer: Pubkey,
}

impl std::fmt::Debug for JsNetworkAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsNetworkAdapter")
            .field("signer", &self.signer)
            .finish()
    }
}

impl JsNetworkAdapter {
    pub fn new(js_network_obj: SolanaNetworkAdapter, signer: Pubkey) -> Self {
        Self {
            js_obj: js_network_obj,
            signer,
        }
    }

    async fn translate_and_send_ordered(
        &self,
        transactions: &[VersionedTransaction],
    ) -> Result<Vec<Signature>, js_sys::Error> {
        let serialized = transactions
            .iter()
            .map(bincode::serialize)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let js_tx_list = js_sys::Array::new_with_length(serialized.len() as u32);

        for (idx, serialized_tx) in serialized.into_iter().enumerate() {
            let buffer = js_sys::Uint8Array::new_with_length(serialized_tx.len() as u32);
            buffer.copy_from(&serialized_tx);

            js_tx_list.set(idx as u32, JsValue::from(buffer));
        }

        let js_results = self.js_obj.send(js_tx_list.into()).await?;
        let results = js_results
            .dyn_into::<js_sys::Array>()?
            .iter()
            .map(|element| {
                element
                    .as_string()
                    .ok_or_else(|| {
                        js_sys::Error::new("send did not return transaction signature string")
                    })
                    .and_then(|string| {
                        Signature::from_str(&string).map_err(|_| {
                            js_sys::Error::new(&format!("could not parse signature: {string}"))
                        })
                    })
            })
            .collect::<Vec<_>>();

        let signatures = results
            .iter()
            .take_while(|r| r.is_ok())
            .map(|r| r.clone().unwrap())
            .collect();
        let error = results.into_iter().find_map(Result::err);

        if let Some(e) = error {
            return Err(e);
        }

        Ok(signatures)
    }
}

#[async_trait(?Send)]
impl NetworkUserInterface for JsNetworkAdapter {
    type Error = js_sys::Error;

    fn signer(&self) -> Pubkey {
        self.signer
    }

    fn get_current_time(&self) -> i64 {
        js_sys::Date::now() as i64
    }

    async fn get_genesis_hash(&self) -> Result<Hash, Self::Error> {
        let js_hash_obj = self.js_obj.get_genesis_hash().await?;
        let Some(hash_string) = js_hash_obj.as_string() else {
            return Err(js_sys::Error::new("genesis hash returned non-string result"));
        };

        Hash::from_str(&hash_string)
            .map_err(|_| js_sys::Error::new("blockhash string not parseble"))
    }

    async fn get_latest_blockhash(&self) -> Result<Hash, Self::Error> {
        let js_hash_obj = self.js_obj.get_latest_blockhash().await?;
        let hash_string = js_reflect_get_string(&js_hash_obj, "blockhash")?;

        Hash::from_str(&hash_string)
            .map_err(|_| js_sys::Error::new("blockhash string not parseble"))
    }

    async fn get_accounts(
        &self,
        addresses: &[Pubkey],
    ) -> Result<Vec<Option<Account>>, Self::Error> {
        let js_address_list = addresses
            .iter()
            .map(|addr| solana_web3::PublicKey::new(addr.as_ref()))
            .collect();
        let js_account_list_obj = self.js_obj.get_accounts(js_address_list).await?;
        let js_account_list = js_account_list_obj
            .dyn_ref::<js_sys::Array>()
            .ok_or_else(|| js_sys::Error::new("did not receive array value from get_accounts"))?;

        js_account_list
            .iter()
            .map(|value| match value.is_truthy() {
                false => Ok(None),
                true => {
                    let lamports = js_reflect_get_u64(&value, "lamports")?;
                    let data = js_reflect_get_bytes(&value, "data")?;
                    let owner = js_reflect_get_pubkey(&value, "owner")?;
                    let rent_epoch = js_reflect_get_u64(&value, "rentEpoch")?;
                    let executable = js_reflect_get_bool(&value, "executable")?;

                    Ok(Some(Account {
                        lamports,
                        data,
                        owner,
                        executable,
                        rent_epoch,
                    }))
                }
            })
            .collect()
    }

    async fn send_ordered(
        &self,
        transactions: &[VersionedTransaction],
    ) -> (Vec<Signature>, Option<Self::Error>) {
        match self.translate_and_send_ordered(transactions).await {
            Ok(signatures) => (signatures, None),

            // FIXME: return successful signatures to caller
            Err(e) => (vec![], Some(e)),
        }
    }

    async fn send_unordered(
        &self,
        transactions: &[VersionedTransaction],
        _blockhash: Option<Hash>,
    ) -> Vec<Result<Signature, Self::Error>> {
        let (signatures, error) = self.send_ordered(transactions).await;

        let mut results = vec![];

        results.extend(signatures.into_iter().map(Ok));
        results.extend(error.map(Err));

        results
    }
}

fn js_reflect_get_string(obj: &JsValue, key: &str) -> Result<String, js_sys::Error> {
    let js_key = JsValue::from_str(key);
    let js_value = Reflect::get(obj, &js_key)?;

    js_value
        .as_string()
        .ok_or_else(|| js_sys::Error::new(&format!("'{key}' is not a string")))
}

fn js_reflect_get_bool(obj: &JsValue, key: &str) -> Result<bool, js_sys::Error> {
    let js_key = JsValue::from_str(key);
    let js_value = Reflect::get(obj, &js_key)?;

    js_value
        .as_bool()
        .ok_or_else(|| js_sys::Error::new(&format!("'{key}' is not a bool")))
}

fn js_reflect_get_u64(obj: &JsValue, key: &str) -> Result<u64, js_sys::Error> {
    let js_key = JsValue::from_str(key);
    let js_value = Reflect::get(obj, &js_key)?;

    if js_value.is_bigint() {
        let integer = js_value.dyn_into::<BigInt>()?;

        u64::try_from(integer).map_err(|value| {
            js_sys::Error::new(&format!(
                "'{key}' is bigint, but not convertible to u64: {value}"
            ))
        })
    } else {
        let float_value = js_value
            .as_f64()
            .ok_or_else(|| js_sys::Error::new(&format!("'{key}' is not a number")))?;
        Ok(float_value as u64)
    }
}

fn js_reflect_get_bytes(obj: &JsValue, key: &str) -> Result<Vec<u8>, js_sys::Error> {
    let js_key = JsValue::from_str(key);
    let js_value = Reflect::get(obj, &js_key)?;

    if let Some(array) = js_value.dyn_ref::<Uint8Array>() {
        Ok(array.to_vec())
    } else {
        Err(js_sys::Error::new("type not a byte buffer/array"))
    }
}

fn js_reflect_get_pubkey(obj: &JsValue, key: &str) -> Result<Pubkey, js_sys::Error> {
    let js_key = JsValue::from_str(key);
    let js_value = Reflect::get(obj, &js_key)?;

    if let Some(js_pubkey) = js_value.dyn_ref::<solana_web3::PublicKey>() {
        let buffer = js_pubkey.to_bytes();

        Ok(Pubkey::try_from(&*buffer).unwrap())
    } else if let Some(array) = js_value.dyn_ref::<Uint8Array>() {
        let mut buffer = [0u8; 32];
        array.copy_to(&mut buffer[..32]);

        Ok(Pubkey::from(buffer))
    } else if let Some(string) = js_value.as_string() {
        Ok(Pubkey::from_str(&string).map_err(|_| js_sys::Error::new("could not parse pubkey"))?)
    } else {
        Err(js_sys::Error::new("type not a pubkey"))
    }
}
