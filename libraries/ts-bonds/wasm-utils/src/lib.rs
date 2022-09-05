#![allow(non_snake_case)]

pub mod orderbook;

use wasm_bindgen::prelude::*;

/// For calculation of limit prices given to the Bonds orderbook
/// 100 at 10% interest -> base: 100, quote: 110
#[wasm_bindgen]
pub fn calculate_limit_price(base: i64, quote: i64) -> u64 {
    let price = base as f64 / quote as f64;
    (price * ((1u64 << 32) as f64)) as u64
}

#[wasm_bindgen]
pub fn MAX_U64() -> u64 {
    u64::MAX
}
