use std::fmt::Display;

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

use super::interest_pricing::fp32_to_f64;

#[wasm_bindgen]
pub struct Order {
    /// Pukbey of the signer allowed to make changes to this order
    #[wasm_bindgen(getter_with_clone)]
    pub owner: Uint8Array,
    /// order tag used to track pdas related to this order
    /// 16 byte hash derived
    #[wasm_bindgen(getter_with_clone)]
    pub order_tag: Uint8Array,
    /// The orderId as found on the orderbook
    /// a u128, used for cancel order instructions
    #[wasm_bindgen(getter_with_clone)]
    pub order_id: Uint8Array,
    /// Total bond ticket worth of the order
    pub base_size: u64,
    /// Total underlying token worth of the order
    pub quote_size: u64,
    /// Fixed point 32 representation of the price
    pub limit_price: u64,
}

/// Represents a 3-tuple of order parameters, returned when calculating order parameters from a given
/// amount and interest rate
#[wasm_bindgen]
pub struct OrderAmount {
    /// max base quantity for an order
    pub base: u64,
    /// max quote quantity for an order
    pub quote: u64,
    /// fixed-point 32 limit price value
    pub price: u64,
}

#[wasm_bindgen(module = "/src/cast.js")]
extern "C" {
    // This will let Rust regain ownership of Order
    #[wasm_bindgen(js_name = castInto)]
    pub fn cast_into_order(value: JsValue) -> Order;
}

impl From<JsValue> for Order {
    fn from(order: JsValue) -> Self {
        cast_into_order(order)
    }
}

#[wasm_bindgen]
impl Order {
    #[wasm_bindgen(constructor)]
    pub fn new(
        owner: Uint8Array,
        order_tag: Uint8Array,
        order_id: Uint8Array,
        base_size: u64,
        quote_size: u64,
        limit_price: u64
    ) -> Order {
        Order {
            owner,
            order_tag,
            order_id,
            base_size,
            quote_size,
            limit_price
        }
    }

    #[wasm_bindgen]
    pub fn owned_by(&self, candidate: &Uint8Array) -> bool {
        let mut a = [0; 32];
        let mut b = [0; 32];

        self.owner.copy_to(&mut a);
        candidate.copy_to(&mut b);

        a == b
    }
}

impl Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Order(quote_size={}, base_size={}, limit_price={})",
            self.quote_size,
            self.base_size,
            fp32_to_f64(self.limit_price),
        )
    }
}
