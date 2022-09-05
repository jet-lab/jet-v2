#![allow(non_snake_case)]

pub mod orderbook;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn MAX_U64() -> u64 {
    u64::MAX
}
