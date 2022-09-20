use jet_proto_math::fixed_point::Fp32;
use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::*;

use super::{critbit::Slab, types::Order};

/// Converts a buffer from an orderbook side into an array of orders on the book
///
/// Params:
///
/// `slab_bytes`: a `UInt8Array` from the AccountInfo data
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
                    owner: Uint8Array::from(&callback.owner[..]),
                    order_tag: Uint8Array::from(&callback.order_tag[..]),
                    base_size: leaf.base_quantity,
                    quote_size: Fp32::upcast_fp32(leaf.price())
                        .u64_mul(leaf.base_quantity)
                        .unwrap(),
                    limit_price: leaf.price(),
                    order_id: Uint8Array::from(&leaf.key.to_le_bytes()[..]),
                }
            })
            .map(JsValue::from),
    )
}

/// Given a base quanity and fixed-point 32 price value, calculate the quote
#[wasm_bindgen]
pub fn base_to_quote(base: u64, price: u64) -> u64 {
    let quote = Fp32::upcast_fp32(price) * base;
    quote.as_u64().unwrap()
}

/// Given a base quanity and fixed-point 32 price value, calculate the quote
#[wasm_bindgen]
pub fn quote_to_base(quote: u64, price: u64) -> u64 {
    let base = Fp32::upcast_fp32(price) / quote;
    base.as_u64().unwrap()
}

/// Given a fixed-point 32 value, convert to decimal representation
#[wasm_bindgen]
pub fn fixed_point_to_decimal(fp: u64) -> u64 {
    Fp32::upcast_fp32(fp).as_u64().unwrap()
}

/// Given an interest rate and bond duration, calculates a price
///
/// NOTE: price is returned in fixed point 32 format
#[wasm_bindgen]
pub fn rate_to_price(_interest_rate: f64, _duration: u64) -> u64 {
    todo!()
}

/// Given a price and bond duration, calculates an interest rate
///
/// NOTE: price is expected to be in fixed point 32 format
#[wasm_bindgen]
pub fn price_to_rate(_price: u64, _duration: u64) -> u64 {
    todo!()
}

/// Converts a fixed point 32 price to an f64 for UI display
#[wasm_bindgen]
pub fn ui_price(_price: u64) -> f64 {
    todo!()
}

/// For calculation of an implied limit price given to the bonds orderbook
///
/// Base is principal plus interest
///
/// Quote is principal
///
/// Example usage
/// ```ignore
/// // 100 token lamports at 10% interest
/// let price = calculate_implied_price(110, 100);
/// ```
#[wasm_bindgen]
pub fn calculate_implied_price(base: u64, quote: u64) -> u64 {
    let price = Fp32::from(quote) / base;
    price.as_u64().unwrap()
}
