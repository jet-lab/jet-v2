use wasm_bindgen::prelude::*;

use solana_sdk::pubkey::Pubkey;

use jet_client::margin_pool::MarginAccountPoolClient;

use crate::ClientError;

#[wasm_bindgen]
pub struct MarginAccountPoolWebClient(pub(crate) MarginAccountPoolClient);

#[wasm_bindgen]
impl MarginAccountPoolWebClient {
    pub async fn deposit(&self, amount: u64, source: Option<Pubkey>) -> Result<(), ClientError> {
        Ok(self.0.deposit(amount, source.as_ref()).await?)
    }

    pub async fn withdraw(
        &self,
        amount: Option<u64>,
        destination: Option<Pubkey>,
    ) -> Result<(), ClientError> {
        Ok(self.0.withdraw(amount, destination.as_ref()).await?)
    }

    #[wasm_bindgen(js_name = borrowWithdraw)]
    pub async fn borrow_withdraw(
        &self,
        amount: Option<u64>,
        destination: Option<Pubkey>,
    ) -> Result<(), ClientError> {
        Ok(self.0.withdraw(amount, destination.as_ref()).await?)
    }

    #[wasm_bindgen(js_name = cancelBorrow)]
    pub async fn cancel_borrow(&self, amount: Option<u64>) -> Result<(), ClientError> {
        Ok(self.0.cancel_borrow(amount).await?)
    }

    #[wasm_bindgen(js_name = depositRepay)]
    pub async fn deposit_repay(
        &self,
        amount: Option<u64>,
        source: Option<Pubkey>,
    ) -> Result<(), ClientError> {
        Ok(self.0.deposit_repay(amount, source.as_ref()).await?)
    }
}
