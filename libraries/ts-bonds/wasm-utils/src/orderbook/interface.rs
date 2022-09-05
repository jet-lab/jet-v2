use js_sys::{Array, Uint8Array};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use super::critbit::Slab;

#[wasm_bindgen]
pub struct Order {
    #[wasm_bindgen(getter_with_clone)]
    pub orderbook_account_key: Uint8Array,
    #[wasm_bindgen(getter_with_clone)]
    pub order_tag: Uint8Array,
    pub base_size: u64,
    pub price: u64,
}

/// Converts a buffer from an orderbook side into an array of orders on the book
/// Params:
///     `slab_bytes` -- a UInt8Array from the AccountInfo data
#[wasm_bindgen]
pub fn get_orders_from_slab(slab_bytes: &[u8]) -> Array {
    let buf = &mut slab_bytes.to_owned();
    let buf_clone = &mut slab_bytes.to_owned();

    let slab = Slab::from_buffer_unchecked(buf).unwrap();
    let slab_clone = Slab::from_buffer_unchecked(buf_clone).unwrap();

    Array::from_iter(
        slab_clone
            .into_iter(true)
            .map(|leaf| {
                let handle = slab.find_by_key(leaf.key).unwrap();
                let callback = slab.get_callback_info(handle);
                Order {
                    orderbook_account_key: Uint8Array::from(&callback.orderbook_account_key[..]),
                    order_tag: Uint8Array::from(&callback.order_tag[..]),
                    base_size: leaf.base_quantity,
                    price: leaf.price(),
                }
            })
            .map(JsValue::from),
    )
}

/// Test order for checking deserialization
#[wasm_bindgen]
pub fn TEST_ORDER() -> Order {
    Order {
        orderbook_account_key: Uint8Array::from(&[1u8; 32][..]),
        order_tag: Uint8Array::from(&[2u8; 16][..]),
        base_size: 3_333,
        price: 4_444,
    }
}

/// Array of orders for testing deserialization
#[wasm_bindgen]
pub fn TEST_ORDER_ARRAY() -> Array {
    Array::from_iter((0..13).map(|_| JsValue::from(TEST_ORDER())))
}
