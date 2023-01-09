// FIXME Export Typescript declarations
use wasm_bindgen::prelude::*;

use crate::core::orderbook::OrderbookModel;

#[wasm_bindgen(js_name = "OrderbookModel", skip_typescript)]
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

#[wasm_bindgen(typescript_custom_section)]
const ORDERBOOKMODEL_RETURN_TYPES: &'static str = r#"
export type Order = {
    owner: any,
    order_tag: any,
    base_size: bigint,
    price: bigint,
}

export type LiquidityObservation = {
    cumulative_base: bigint,
    cumulative_quote: bigint,
    cumulative_price: number,
    cumulative_rate: number,
};

export type LiquiditySample = {
    side: any,
    total_quote_qty: bigint,
    sample_quote_qty: bigint,
    points: Array<LiquidityObservation>,
}

export type FillSimulation = {
    limit_price: number,
    order_quote_qty: bigint,
    filled_quote_qty: bigint,
    unfilled_quote_qty: bigint,
    filled_base_qty: bigint,
    matches: usize,
    vwap: number,
    vwar: number,
    fills: Array<Fill>,
}

export type Fill = {
    base_qty: bigint,
    quote_qty: bigint,
    price: number,
}

export type QueuingSimulation = {
    depth: bigint,
    preceding_base_qty: bigint,
    preceding_quote_qty: bigint,
    vwap: number,
    vwar: number,
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const ORDERBOOKMODEL_CLAS: &'static str = r#"
/**
*/
export class OrderbookModel {
    free(): void;
  /**
  * @param {bigint} tenor
  */
    constructor(tenor: bigint);
  /**
  * @param {Uint8Array} bidsBuffer
  * @param {Uint8Array} asksBuffer
  */
    refresh(bidsBuffer: Uint8Array, asksBuffer: Uint8Array): void;
  /**
  * @param {string} side
  * @returns {LiquiditySample}
  */
    sampleLiquidity(side: string): LiquiditySample;
  /**
  * @param {string} action
  * @param {bigint} limitPrice
  * @returns {boolean}
  */
    wouldMatch(action: string, limitPrice: bigint): boolean;
  /**
  * @param {string} action
  * @param {bigint} quoteQty
  * @param {bigint | undefined} limitPrice
  * @returns {FillSimulation}
  */
    simulateFills(action: string, quoteQty: bigint, limitPrice?: bigint): FillSimulation;
  /**
  * @param {string} action
  * @param {bigint} limitPrice
  * @returns {QueuingSimulation}
  */
    simulateQueuing(action: string, limitPrice: bigint): QueuingSimulation;
  /**
  * @returns {Array<Order>}
  */
    loanOffers(): Array<Order>;
  /**
  * @returns {Array<Order>}
  */
    loanRequests(): Array<Order>;
  }
"#;
