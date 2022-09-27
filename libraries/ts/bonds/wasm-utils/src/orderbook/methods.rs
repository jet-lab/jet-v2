use jet_proto_math::fixed_point::Fp32;
use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::*;

use super::{
    critbit::Slab,
    interest_pricing::{fp32_to_f64, InterestPricer, PricerImpl},
    types::Order,
};

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

/// Given a price and bond duration, calculates an interest rate
///
/// price: underlying per bond ticket: fixed point 32 (left shifted 32 bits to get fractional precision)
/// tenor: seconds
/// return: interest rate in basis points
#[wasm_bindgen]
pub fn price_to_rate(price: u64, tenor: u64) -> u64 {
    PricerImpl::price_fp32_to_bps_yearly_interest(price, tenor)
}

/// Given an interest rate and bond duration, calculates a price
///
/// interest_rate: basis points
/// tenor: seconds
/// return: price: underlying per bond ticket: fixed point 32 (left shifted 32 bits to get fractional precision)
#[wasm_bindgen]
pub fn rate_to_price(interest_rate: u64, tenor: u64) -> u64 {
    PricerImpl::yearly_interest_bps_to_fp32_price(interest_rate, tenor)
}

/// Converts a fixed point 32 price to an f64 for UI display
#[wasm_bindgen]
pub fn ui_price(price: u64) -> f64 {
    fp32_to_f64(price)
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

/// This is meant to ensure that the api is using the PricerImpl type alias,
/// rather than circumventing it to use some other implementation. A lot of the
/// tests are written against PricerImpl so this ensures the api is well tested.
/// To change the implementation for the wasm bindings, change which type the
/// alias PricerImpl points to. Don't directly use an InterestPricer
/// implementation in the wasm bindings.
#[test]
fn wasm_uses_tested_implementation() {
    for tenor in 1..100u64 {
        for printerice in 1 << 10..1 << 13 {
            let price = printerice << 19;
            let tenor = tenor.pow(3);
            assert_eq!(
                PricerImpl::price_fp32_to_bps_yearly_interest(price, tenor),
                price_to_rate(price, tenor)
            );
            assert_eq!(
                PricerImpl::yearly_interest_bps_to_fp32_price(printerice, tenor),
                rate_to_price(printerice, tenor)
            );
        }
    }
}
