use jet_program_common::interest_pricing::f64_to_bps;
use wasm_bindgen::prelude::*;

use jet_client::fixed_term::MarginAccountMarketClient;

use crate::ClientError;

#[wasm_bindgen]
pub struct MarginAccountFixedTermMarketWebClient(pub(crate) MarginAccountMarketClient);

#[wasm_bindgen]
impl MarginAccountFixedTermMarketWebClient {
    #[wasm_bindgen(js_name = offerLoan)]
    pub async fn offer_loan(&self, amount: u64, rate: f64) -> Result<(), ClientError> {
        let rate_bps = f64_to_bps(rate);
        Ok(self.0.offer_loan(amount, rate_bps).await?)
    }

    #[wasm_bindgen(js_name = requestLoan)]
    pub async fn request_loan(&self, amount: u64, rate: f64) -> Result<(), ClientError> {
        let rate_bps = f64_to_bps(rate);
        Ok(self.0.request_loan(amount, rate_bps).await?)
    }

    #[wasm_bindgen(js_name = lendNow)]
    pub async fn lend_now(&self, amount: u64) -> Result<(), ClientError> {
        Ok(self.0.lend_now(amount).await?)
    }

    #[wasm_bindgen(js_name = borrowNow)]
    pub async fn borrow_now(&self, amount: u64) -> Result<(), ClientError> {
        Ok(self.0.borrow_now(amount).await?)
    }

    #[wasm_bindgen(js_name = repay)]
    pub async fn repay(&self, max_amount: u64) -> Result<(), ClientError> {
        Ok(self.0.repay(max_amount).await?)
    }

    #[wasm_bindgen(js_name = redeemDeposits)]
    pub async fn redeem_deposits(&self) -> Result<(), ClientError> {
        Ok(self.0.redeem_deposits().await?)
    }

    #[wasm_bindgen(js_name = settle)]
    pub async fn settle(&self) -> Result<(), ClientError> {
        Ok(self.0.settle().await?)
    }
}
