mod interest_pricing;
pub mod methods;
pub mod types;

#[cfg(feature = "testing")]
pub mod test;

use agnostic_orderbook::state::critbit::Slab;
use jet_fixed_term::orderbook::state::CallbackInfo;
use jet_program_common::Fp32;
use js_sys::Uint8Array;
use js_sys::Array;
use wasm_bindgen::prelude::*;

use self::types::Order;

#[wasm_bindgen(module = "src/store/src/type.ts")]
extern "C" {
    type OpenOrder;
}

#[wasm_bindgen]
pub struct OrderBook {
    bids: Vec<Order>,
    asks: Vec<Order>,
}

#[wasm_bindgen]
impl OrderBook {
    // pub fn refresh(&mut self, bids_buffer: &[u8], asks_buffer: &[u8]) {
    //     let x: OpenOrder = OpenOrder { obj:  }
    // }

    pub fn bids(&self) -> Array { // TODO: Find a way to specify TS type as Array<Order>
        Array::from_iter(self.bids.clone().into_iter().map(JsValue::from))
    }

    pub fn asks(&self) -> Array { // TODO: Find a way to specify TS type as Array<Order>
        Array::from_iter(self.asks.clone().into_iter().map(JsValue::from))
    }
}

/// Converts a buffer from an orderbook side into an array of orders on the book
///
/// Params:
///
/// `slab_bytes`: a `UInt8Array` from the AccountInfo data
#[wasm_bindgen]
pub fn get_orders_from_slab(slab_bytes: &[u8]) -> Array {
    let buf = &mut slab_bytes.to_owned();
    let buf_clone = &mut slab_bytes.to_owned();

    let slab: Slab<CallbackInfo> = Slab::from_buffer_unchecked(buf).unwrap();
    let slab_clone: Slab<CallbackInfo> = Slab::from_buffer_unchecked(buf_clone).unwrap();

    Array::from_iter(
        slab_clone
            .into_iter(true)
            .map(|leaf| {
                let handle = slab.find_by_key(leaf.key).unwrap();
                let callback = slab.get_callback_info(handle);
                Order {
                    owner: Uint8Array::from(&callback.owner.to_bytes()[..]),
                    order_tag: Uint8Array::from(&callback.order_tag.bytes()[..]),
                    base_size: leaf.base_quantity,
                    quote_size: Fp32::upcast_fp32(leaf.price())
                        .decimal_u64_mul(leaf.base_quantity)
                        .unwrap(),
                    limit_price: leaf.price(),
                    order_id: Uint8Array::from(&leaf.key.to_le_bytes()[..]),
                }
            })
            .map(JsValue::from),
    )
}
