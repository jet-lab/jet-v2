use jet_bonds_lib::utils::OrderAmount;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Order {
    /// Account pubkey asssociated with this order
    #[wasm_bindgen(getter_with_clone)]
    pub account_key: Uint8Array,
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
    pub price: u64,
}

#[wasm_bindgen(js_name = OrderAmount)]
pub struct JsOrderAmount {
    pub base: u64,
    pub quote: u64,
    pub price: u64,
}

impl<T: Into<OrderAmount>> From<T> for JsOrderAmount {
    fn from(amount: T) -> JsOrderAmount {
        let o: OrderAmount = amount.into();
        JsOrderAmount {
            base: o.base,
            quote: o.quote,
            price: o.price,
        }
    }
}
