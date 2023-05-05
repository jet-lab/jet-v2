use serde_wasm_bindgen::Serializer;
use solana_program::pubkey::Pubkey;
use wasm_bindgen::prelude::*;

use crate::core::orderbook::OrderbookModel;

#[wasm_bindgen(js_name = "OrderbookModel", skip_typescript)]
pub struct JsOrderbookModel {
    inner: OrderbookModel,
    serializer: Serializer,
}

impl JsOrderbookModel {
    fn get_serializer() -> Serializer {
        Serializer::new().serialize_large_number_types_as_bigints(true)
    }

    fn to_js<T>(&self, value: &T) -> Result<JsValue, JsError>
    where
        T: serde::ser::Serialize + ?Sized,
    {
        Ok(value.serialize(&self.serializer)?)
    }
}

#[wasm_bindgen(js_class = "OrderbookModel")]
impl JsOrderbookModel {
    #[wasm_bindgen(constructor)]
    pub fn new(tenor: u64) -> Self {
        Self {
            inner: OrderbookModel::new(tenor),
            serializer: Self::get_serializer(),
        }
    }

    #[allow(non_snake_case)]
    pub fn refresh(&mut self, bidsBuffer: &[u8], asksBuffer: &[u8]) {
        self.inner.refresh(bidsBuffer, asksBuffer);
    }

    #[wasm_bindgen(js_name = "refreshFromSnapshot")]
    pub fn refresh_from_snapshot(&mut self, snapshot: JsValue) {
        let snapshot = serde_wasm_bindgen::from_value(snapshot).unwrap();
        self.inner.refresh_from_snapshot(snapshot);
    }

    #[wasm_bindgen(js_name = "sampleLiquidityDeprecated")]
    pub fn sample_liquidity(&self, side: &str) -> Result<JsValue, JsError> {
        let sample = self.inner.sample_liquidity(side.into(), None, None); // TODO try_into

        self.to_js(&sample)
    }

    #[wasm_bindgen(js_name = "sampleLiquidity")]
    pub fn sample_liquidity_v2(&self, max_quote_qty: u64) -> Result<JsValue, JsError> {
        let sample = self.inner.sample_liquidity_v2(max_quote_qty);

        self.to_js(&sample)
    }

    #[wasm_bindgen(js_name = "wouldMatch")]
    #[allow(non_snake_case)]
    pub fn would_match(&self, action: &str, limitPrice: u64) -> bool {
        self.inner.would_match(action.into(), limitPrice) // TODO try_into
    }

    #[wasm_bindgen(js_name = "simulateTaker")]
    #[allow(non_snake_case)]
    pub fn simulate_taker(
        &self,
        action: &str,
        quoteQty: u64,
        limitPrice: Option<u64>,
        user: Option<Box<[u8]>>,
    ) -> Result<JsValue, JsError> {
        let user = user.map(|b| Pubkey::new(&b));
        let sim = self
            .inner
            .simulate_taker(action.into(), quoteQty, limitPrice, user); // TODO try_into

        self.to_js(&sim)
    }

    #[wasm_bindgen(js_name = "simulateMaker")]
    #[allow(non_snake_case)]
    pub fn simulate_maker(
        &self,
        action: &str,
        quoteQty: u64,
        limitPrice: u64,
        user: Option<Box<[u8]>>,
    ) -> Result<JsValue, JsError> {
        let user = user.map(|b| Pubkey::new(&b));
        let sim = self
            .inner
            .simulate_maker(action.into(), quoteQty, limitPrice, user);

        self.to_js(&sim)
    }

    #[wasm_bindgen(js_name = "loanOffers")]
    pub fn loan_offers(&self) -> Result<JsValue, JsError> {
        self.to_js(&self.inner.loan_offers())
    }

    #[wasm_bindgen(js_name = "loanRequests")]
    pub fn loan_requests(&self) -> Result<JsValue, JsError> {
        self.to_js(&self.inner.loan_requests())
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

export type OrderbookSnapshot = {
    bids: Array<Order>,
    asks: Array<Order>,
};

export type LiquidityObservation = {
    cumulative_base: bigint,
    cumulative_quote: bigint,
    cumulative_price: number,
    cumulative_rate: number,
};

export type LiquiditySampleDeprecated = {
    side: any,
    total_quote_qty: bigint,
    sample_quote_qty: bigint,
    points: Array<LiquidityObservation>,
}

export type LiquiditySample = {
    base: any,
    quote: any,
    bids: Array<[number, bigint]>,
    asks: Array<[number, bigint]>,
    price_range: [number, number],
    liquidity_range: [bigint, bigint],
}

export type TakerSimulation = {
    order_quote_qty: bigint,
    limit_price: number,

    would_match: bool,
    self_match: bool,
    matches: bigint,
    filled_quote_qty: bigint,
    filled_base_qty: bigint,
    filled_vwap: number,
    filled_vwar: number,
    fills: Array<Fill>,

    unfilled_quote_qty: bigint,
}

export type Fill = {
    base_qty: bigint,
    quote_qty: bigint,
    price: number,
}

export type MakerSimulation = {
    order_quote_qty: bigint,
    limit_price: number,

    full_quote_qty: bigint,
    full_base_qty: bigint,
    full_vwap: number,
    full_vwar: number,

    would_post: bool,
    depth: bigint,
    posted_quote_qty: bigint,
    posted_base_qty: bigint,
    posted_vwap: number,
    posted_vwar: number,
    preceding_base_qty: bigint,
    preceding_quote_qty: bigint,
    preceding_vwap: number,
    preceding_vwar: number,

    would_match: bool,
    self_match: bool,
    matches: bigint,
    filled_quote_qty: bigint,
    filled_base_qty: bigint,
    filled_vwap: number,
    filled_vwar: number,
    fills: Array<Fill>,
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
    * @param {OrderbookSnapshot} snapshot
    */
    refreshFromSnapshot(snapshot: OrderbookSnapshot): void;
    /**
    * @param {string} side
    * @returns {LiquiditySampleDeprecated}
    */
    sampleLiquidityDeprecated(side: string): LiquiditySampleDeprecated;
    /**
    * @param {bigint} maxQuoteQty
    * @returns {LiquiditySample}
    */
    sampleLiquidity(maxQuoteQty: u64): LiquiditySample;
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
    * @param {Uint8Array | undefined} user
    * @returns {TakerSimulation}
    */
    simulateTaker(action: string, quoteQty: bigint, limitPrice?: bigint, user?: Uint8Array): TakerSimulation;
    /**
    * @param {string} action
    * @param {bigint} quoteQty
    * @param {bigint} limitPrice
    * @param {Uint8Array | undefined} user
    * @returns {MakerSimulation}
    */
    simulateMaker(action: string, quoteQty: bigint, limitPrice: bigint, user?: Uint8Array): MakerSimulation;
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
