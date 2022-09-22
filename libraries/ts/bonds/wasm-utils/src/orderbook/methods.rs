use jet_proto_math::{
    fixed_point::{Fp32, FP32_ONE},
    number::Number,
};
use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::*;

use super::{critbit::Slab, types::Order};

const SECONDS_PER_YEAR: u64 = 31_536_000;

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
                        .decimal_u64_mul(leaf.base_quantity)
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
    quote.as_decimal_u64().unwrap()
}

/// Given a base quanity and fixed-point 32 price value, calculate the quote
#[wasm_bindgen]
pub fn quote_to_base(quote: u64, price: u64) -> u64 {
    let base = Fp32::upcast_fp32(price) / quote;
    base.as_decimal_u64().unwrap()
}

/// Given a fixed-point 32 value, convert to decimal representation
#[wasm_bindgen]
pub fn fixed_point_to_decimal(fp: u64) -> u64 {
    Fp32::upcast_fp32(fp).as_decimal_u64().unwrap()
}

/// Given an interest rate and bond duration, calculates a price
///
/// Interest rate is given as basis points. Tenor is in seconds.
///
/// NOTE: price is returned in fixed point 32 representation
#[wasm_bindgen]
pub fn rate_to_price(interest_rate: u64, tenor: u64) -> u64 {
    let year_proportion = Number::from(tenor) / SECONDS_PER_YEAR;
    let rate = Number::from(interest_rate) / 10_000;
    let price = (Number::ONE / (Number::ONE + rate * year_proportion)) * FP32_ONE;
    Fp32::wrap_u128(price.as_u128(0)).downcast_u64().unwrap()
}

// rate  = (1 - price) / tenor * price

/// Given a price and bond duration, calculates an interest rate
///
/// Tenor is in seconds, returns an interest rate in basis points
///
/// NOTE: price is expected to be in fixed point 32 representation
#[wasm_bindgen]
pub fn price_to_rate(price: u64, tenor: u64) -> u64 {
    let year_proportion = Number::from(tenor) / SECONDS_PER_YEAR;
    let price = Number::from(price) / FP32_ONE; // convert to decimal representation
    let rate = (Number::ONE - price) / year_proportion * price;
    (rate * 10_000).as_u64(0)
}

/// Converts a fixed point 32 price to an f64 for UI display
#[wasm_bindgen]
pub fn ui_price(price: u64) -> f64 {
    price as f64 / FP32_ONE as f64
}

#[wasm_bindgen]
pub fn build_order_amount_deprecated(amount: u64, interest_rate: u64) -> super::types::OrderAmount {
    let quote = amount;
    let base = quote + ((quote * interest_rate) / 10_000);
    let price = calculate_implied_price(base, quote);

    super::types::OrderAmount { base, quote, price }
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
    price.as_decimal_u64().unwrap()
}

#[cfg(test)]
mod test {
    /// TODO:
    // (rate, tenor) -> price
    // price = 1 / (1 + rate * tenor)
    // price * (1 + rate * tenor) = 1
    // tenor: as fraction of the period. Period is always annual
    // let price = x;
    // assert(price == rate_to_price(price_to_rate(price, tenor), tenor));
    use crate::orderbook::methods::{price_to_rate, rate_to_price};

    #[test]
    fn conversions() {
        use rand::RngCore;

        let mut rng = rand::thread_rng();
        let nums: Vec<_> = (0..1024)
            .map(|_| {
                let x: u64 = rng.next_u64() % 10_000;
                let y: u64 = rng.next_u64() % 10_000_000;
                (x, y)
            })
            .collect();
        for (rate, tenor) in nums {
            assert_eq!(rate, price_to_rate(rate_to_price(rate, tenor), tenor))
        }
    }
}
