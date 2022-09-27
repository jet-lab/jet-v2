use jet_bonds_lib::utils::OrderAmount;
use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::*;

use crate::orderbook::types::JsOrderAmount;

use super::types::Order;

/// Test order for checking deserialization
#[wasm_bindgen]
pub fn TEST_ORDER() -> Order {
    Order {
        account_key: Uint8Array::from(&[1u8; 32][..]),
        order_tag: Uint8Array::from(&[2u8; 16][..]),
        order_id: Uint8Array::from(&3_333_u128.to_le_bytes()[..]),
        base_size: 4_444,
        quote_size: 5_555,
        price: 6_666,
    }
}

/// Array of orders for testing deserialization
#[wasm_bindgen]
pub fn TEST_ORDER_ARRAY() -> Array {
    Array::from_iter((0..13).map(|_| JsValue::from(TEST_ORDER())))
}

/// Test OrderAmount for testing deserialization
#[wasm_bindgen]
pub fn TEST_ORDER_AMOUNT() -> JsOrderAmount {
    OrderAmount::new(100, 1_500).unwrap().into()
}
