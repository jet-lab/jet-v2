// TODO: deduplicate with packages/margin/src/wasm-src/orderbook/methods.rs

use std::ops::Div;

use jet_program_common::{
    interest_pricing::{f64_to_fp32, fp32_to_f64, InterestPricer, PricerImpl},
    Fp32,
};

/// Given a base quanity and fixed-point 32 price value, calculate the quote
pub fn base_to_quote(base: u64, price: u64) -> u64 {
    let quote = Fp32::upcast_fp32(price) * base;
    quote.as_decimal_u64().unwrap()
}

/// Given a base quanity and fixed-point 32 price value, calculate the quote
pub fn quote_to_base(quote: u64, price: u64) -> u64 {
    Fp32::upcast_fp32(price).u64_div(quote).unwrap()
}

/// Given a fixed-point 32 value, convert to decimal representation
pub fn fixed_point_to_decimal(fp: u64) -> u64 {
    Fp32::upcast_fp32(fp).as_decimal_u64().unwrap()
}

/// Given a price and bond duration, calculates an interest rate
///
/// price: underlying per bond ticket: fixed point 32 (left shifted 32 bits to get fractional precision)
/// tenor: seconds
/// return: interest rate in basis points
pub fn price_to_rate(price: u64, tenor: u64) -> u64 {
    PricerImpl::price_fp32_to_bps_yearly_interest(price, tenor)
}

/// Given an interest rate and bond duration, calculates a price
///
/// interest_rate: basis points
/// tenor: seconds
/// return: price: underlying per bond ticket: fixed point 32 (left shifted 32 bits to get fractional precision)
pub fn rate_to_price(interest_rate: u64, tenor: u64) -> u64 {
    PricerImpl::yearly_interest_bps_to_fp32_price(interest_rate, tenor)
}

/// Converts a fixed point 32 price to an f64 for UI display
pub fn ui_price(price: u64) -> f64 {
    fp32_to_f64(price)
}

/// Converts from f64 to a fixed point 32 price
pub fn f64_to_price(price: f64) -> u64 {
    f64_to_fp32(price)
}

/// For calculation of an implied limit price given to the FixedTerm orderbook
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
pub fn calculate_implied_price(base: u64, quote: u64) -> u64 {
    let price = Fp32::from(quote).div(Fp32::from(base));
    price.downcast_u64().unwrap() // FIXME don't panic
}

#[test]
fn test_calculate_implied_price() {
    let result = calculate_implied_price(1000_u64, 1100_u64);
    assert_eq!(result, 4724464025) // FIXME Check this test
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
