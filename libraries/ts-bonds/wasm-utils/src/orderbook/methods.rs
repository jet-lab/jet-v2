use jet_bonds_lib::utils::{Fp32, OrderAmount};
use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::*;

use crate::orderbook::types::JsOrderAmount;

use super::{critbit::Slab, types::Order};

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
                    account_key: Uint8Array::from(&callback.account_key[..]),
                    order_tag: Uint8Array::from(&callback.order_tag[..]),
                    base_size: leaf.base_quantity,
                    quote_size: Fp32::upcast_fp32(leaf.price())
                        .u64_mul(leaf.base_quantity)
                        .unwrap(),
                    price: leaf.price(),
                    order_id: Uint8Array::from(&leaf.key.to_le_bytes()[..]),
                }
            })
            .map(JsValue::from),
    )
}

/// Calculates an `OrderAmount` given an amount being traded and a desired interest rate
#[wasm_bindgen]
pub fn calculate_order_amount(amount: u64, interest: u64) -> JsOrderAmount {
    OrderAmount::new(amount, interest).unwrap().into()
}

/// For calculation of limit prices given to the Bonds orderbook
///
/// Base is principal plus interest
///
/// Quote is principal
///
/// Example usage
/// ```no_run
/// // 100 token lamports at 10% interest
/// calculate_price(110, 100);
/// ```
#[wasm_bindgen]
pub fn calculate_price(base: u64, quote: u64) -> u64 {
    OrderAmount::price(base, quote).unwrap()
}
