use wasm_bindgen::prelude::*;

use solana_sdk::pubkey::Pubkey;

use jet_client::margin::{MarginAccountClient, MarginClient, MarginPosition};

use crate::{
    fixed_term::MarginAccountFixedTermMarketWebClient, margin_pool::MarginAccountPoolWebClient,
    ClientError, JsNetworkAdapter,
};

#[wasm_bindgen]
pub struct MarginWebClient(pub(crate) MarginClient<JsNetworkAdapter>);

#[wasm_bindgen]
impl MarginWebClient {
    pub fn accounts(&self) -> Result<JsValue, ClientError> {
        let accounts = self
            .0
            .accounts()
            .into_iter()
            .map(|inner| JsValue::from(MarginAccountWebClient { inner }));

        Ok(js_sys::Array::from_iter(accounts).into())
    }

    #[wasm_bindgen(js_name = createAccount)]
    pub async fn create_account(&self) -> Result<(), ClientError> {
        Ok(self.0.create_account().await?)
    }
}

#[wasm_bindgen]
pub struct MarginAccountWebClient {
    inner: MarginAccountClient<JsNetworkAdapter>,
}

#[wasm_bindgen(typescript_custom_section)]
const _TS_TYPE_MARGIN_POSITION_LIST: &'static str = r#"
type MarginPositionList = MarginPosition[];
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "MarginPositionList")]
    pub type MarginPositionList;
}

#[wasm_bindgen]
impl MarginAccountWebClient {
    pub fn address(&self) -> Pubkey {
        self.inner.address()
    }

    pub fn airspace(&self) -> Pubkey {
        self.inner.airspace()
    }

    pub fn positions(&self) -> Result<MarginPositionList, ClientError> {
        let positions = self.inner.positions();
        let result = js_sys::Array::new_with_length(positions.len() as u32);

        for (index, position) in positions.into_iter().enumerate() {
            result.set(index as u32, position.into());
        }

        Ok(MarginPositionList::from(JsValue::from(result)))
    }

    pub fn position(&self, token: &Pubkey) -> Result<Option<MarginPosition>, ClientError> {
        Ok(self
            .inner
            .positions()
            .into_iter()
            .find(|p| p.token == *token))
    }

    #[wasm_bindgen(js_name = lendingPool)]
    pub fn lending_pool(&self, token: &Pubkey) -> MarginAccountPoolWebClient {
        MarginAccountPoolWebClient(self.inner.pool(token))
    }

    #[wasm_bindgen(js_name = fixedTermMarket)]
    pub fn fixed_term_market(
        &self,
        market: &Pubkey,
    ) -> Result<MarginAccountFixedTermMarketWebClient, ClientError> {
        Ok(MarginAccountFixedTermMarketWebClient(
            self.inner.fixed_term(market)?,
        ))
    }

    pub async fn deposit(
        &self,
        token: &Pubkey,
        amount: u64,
        source: Option<Pubkey>,
    ) -> Result<(), ClientError> {
        Ok(self.inner.deposit(token, amount, source.as_ref()).await?)
    }

    pub async fn withdraw(
        &self,
        token: &Pubkey,
        amount: u64,
        destination: Option<Pubkey>,
    ) -> Result<(), ClientError> {
        Ok(self
            .inner
            .withdraw(token, amount, destination.as_ref())
            .await?)
    }
}
