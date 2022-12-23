pub mod orderbook;

use wasm_bindgen::prelude::*;

/// Initialise the wasm module. Idempotent.
/// NOTE FIXME despite the start attribute this does not appear to
/// be called on module import.
#[wasm_bindgen(start, js_name = initModule)]
pub fn init_module() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => ($crate::log(&format_args!($($t)*).to_string()))
}
