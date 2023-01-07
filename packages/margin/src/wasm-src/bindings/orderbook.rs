// FIXME Export Typescript declarations
use wasm_bindgen::prelude::*;

use crate::core::orderbook::OrderbookModel;

#[wasm_bindgen(js_name = "OrderbookModel")]
pub struct JsOrderbookModel(OrderbookModel);

// FIXME Ensure serialisation of u64 as bigint
#[wasm_bindgen(js_class = "OrderbookModel")]
impl JsOrderbookModel {
    #[wasm_bindgen(constructor)]
    pub fn new(tenor: u64) -> Self {
        Self(OrderbookModel::new(tenor))
    }

    #[allow(non_snake_case)]
    pub fn refresh(&mut self, bidsBuffer: &[u8], asksBuffer: &[u8]) {
        self.0.refresh(bidsBuffer, asksBuffer);
    }

    #[wasm_bindgen(js_name = "sampleLiquidity")]
    pub fn sample_liquidity(&self, side: &str) -> Result<JsValue, JsError> {
        let sample = self.0.sample_liquidity(side.into()); // TODO try_into

        Ok(serde_wasm_bindgen::to_value(&sample)?)
    }

    #[wasm_bindgen(js_name = "wouldMatch")]
    #[allow(non_snake_case)]
    pub fn would_match(&self, action: &str, limitPrice: u64) -> bool {
        self.0.would_match(action.into(), limitPrice) // TODO try_into
    }

    #[wasm_bindgen(js_name = "simulateFills")]
    #[allow(non_snake_case)]
    pub fn simulate_fills(
        &self,
        action: &str,
        quoteQty: u64,
        limitPrice: Option<u64>,
    ) -> Result<JsValue, JsError> {
        let sim = self.0.simulate_fills(action.into(), quoteQty, limitPrice); // TODO try_into

        Ok(serde_wasm_bindgen::to_value(&sim)?)
    }

    #[wasm_bindgen(js_name = "simulateQueuing")]
    #[allow(non_snake_case)]
    pub fn simulate_queuing(&self, action: &str, limitPrice: u64) -> Result<JsValue, JsError> {
        let sim = self.0.simulate_queuing(action.into(), limitPrice);

        Ok(serde_wasm_bindgen::to_value(&sim)?)
    }

    #[wasm_bindgen(js_name = "loanOffers")]
    pub fn loan_offers(&self) -> Result<JsValue, JsError> {
        Ok(serde_wasm_bindgen::to_value(self.0.loan_offers())?)
    }

    #[wasm_bindgen(js_name = "loanRequests")]
    pub fn loan_requests(&self) -> Result<JsValue, JsError> {
        Ok(serde_wasm_bindgen::to_value(self.0.loan_requests())?)
    }
}
