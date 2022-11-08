use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

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

#[wasm_bindgen(module = "/src/orderbook/cast.js")]
extern "C" {
  // This will let Rust regain ownership of `Foo`
  #[wasm_bindgen(js_name = castInto)]
  pub fn cast_into_order(value: JsValue) -> Order;
}

impl From<JsValue> for Order {
    fn from(order: JsValue) -> Self {
        cast_into_order(order)
    }
}

impl Order {
    pub fn owned_by(&self, candidate: &Uint8Array) -> bool {
        let mut a = [0; 32];
        let mut b = [0; 32];

        self.owner.copy_to(&mut a);
        candidate.copy_to(&mut b);

        a == b
    }
}
