#![allow(non_snake_case)]

mod error;
mod interest_pricing;
pub mod methods;
pub mod orderbook;
#[cfg(feature = "testing")]
pub mod test;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn MAX_U64() -> u64 {
    u64::MAX
}

#[macro_export]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ($crate::log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
