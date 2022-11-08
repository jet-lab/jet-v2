// FIXME Don't panic in wasm


use crate::console_log;
use crate::log;

use super::{error::FixedTermWasmError, interest_pricing::f64_to_fp32};
use jet_program_common::Fp32;
use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::*;

extern crate console_error_panic_hook;

use super::{
    critbit::Slab,
    interest_pricing::{fp32_to_f64, InterestPricer, PricerImpl},
    types::Order,
};

pub type Result<T> = std::result::Result<T, JsError>;

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

/// Given some bytes, reconstruct the u128 order_id and pass it back as a string
#[wasm_bindgen]
pub fn order_id_to_string(order_id: &[u8]) -> String {
    u128::from_le_bytes(order_id.try_into().unwrap()).to_string()
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

    // price ~ quote per base
    // base ~ quote / price
    // Fp32::upcast_fp32(price).u64_div(quote).unwrap()

    (Fp32::ONE / Fp32::upcast_fp32(price) * quote).as_decimal_u64().unwrap() // FIXME Check floor or ceil
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
    (Fp32::from(quote) / base).downcast_u64().unwrap() // FIXME panic
}

/// Identifies the role of the user.
#[wasm_bindgen]
pub enum Actor {
    Lender,
    Borrower,
}

/// Actions that an `Actor` may be taking.
#[wasm_bindgen]
#[derive(PartialEq, Eq)]
pub enum Action {
    RequestLoan,
    RequestBorrow,
    LendNow,
    BorrowNow,
}

/// Adjusts a price to tolerate an amonut of slipapge.
///
/// price is the limit price of an order; FP32 representation.
/// fraction is the amount of slippage, eg 0.05 would be 5%.
/// trader is the party constructing the order.
///
/// Returns the adjusted price in FP32 representation.
#[wasm_bindgen]
pub fn with_slippage(price: u64, fraction: f64, actor: Actor) -> u64 {
    let scale = match actor {
        Actor::Lender => 1_f64 + fraction,
        Actor::Borrower => 1_f64 - fraction,
    };

    f64_to_fp32(fp32_to_f64(price) * scale)
}

/// The estimated result of emitting an order into the orderbook, as produced
/// by `estimated_order_outcome`.
#[wasm_bindgen]
pub struct EstimatedOrderOutcome {
    /// The approxiamte volume-weighted average price achieved, FP32 representation.
    pub vwap: u64,
    /// The number of quote lamports filled.
    pub filled_quote: u64,
    /// The number of base lamports filled.
    pub filled_base: u64,
    /// The number of requested quote lamports left unfilled.
    pub unfilled_quote: u64,
    /// The number of matches contributing to the result.
    pub matches: u32,
}

/// Estimates the outcome of a hypothetical order.
///
/// quote_size is the amount of quote lamports the order seeks to trade.
/// taker is the would-be owner of the order.
/// order_type categories the would be order.
/// limit_price is required for limit order types; FP32 representation.
/// resting_orders is an array of orders that could be hit by the taker order. The "opposite"
///     side of the book from the taker, so to speak. These must be ordered appropriately
///     or an error will be returned, ie sorted ascending (descending) by limit price for
///     asks (bids).
#[wasm_bindgen]
pub fn estimate_order_outcome(
    quote_size: u64,
    taker: Uint8Array,
    order_type: Action,
    limit_price: Option<u64>,
    resting_orders: Array,
) -> Result<EstimatedOrderOutcome> {
    console_error_panic_hook::set_once();

    let mut unfilled_quote = quote_size;
    let mut filled_quote = 0_u64;
    let mut filled_base = 0_u64;
    let mut matches = 0_u32;
    let mut last_price = match order_type {
        // To enforce ordering of resting orders
        Action::RequestLoan | Action::LendNow => u64::MIN,
        Action::RequestBorrow | Action::BorrowNow => u64::MAX,
    };
    let limit_price = match limit_price {
        Some(p) => p,
        None => {
            if order_type == Action::RequestBorrow || order_type == Action::RequestLoan {
                return Err(FixedTermWasmError::LimitPriceRequired.into());
            } else {
                0_u64
            }
        }
    };

    console_log!(
        "Simulating matches with limit price {} and quote size: {}",
        fp32_to_f64(limit_price),
        quote_size,
    );

    for item in resting_orders.iter() {
        if unfilled_quote == 0 {
            break;
        }

        let order: Order = item.clone().into();

        console_log!("Order {}: {}", matches, order);

        // Ensure that we're processing appropriately ordered resting_orders. The correct
        // ordering depends on whether the hypothetical order being processed is hitting
        // bids or asks.

        match order_type {
            Action::RequestLoan | Action::LendNow => {
                if order.limit_price < last_price {
                    return Err(FixedTermWasmError::RestingOrdersNotSorted.into());
                }
            }
            Action::RequestBorrow | Action::BorrowNow => {
                if order.limit_price > last_price {
                    return Err(FixedTermWasmError::RestingOrdersNotSorted.into());
                }
            }
        };
        last_price = order.limit_price;

        // For limit orders we might be done matching.

        if (order_type == Action::RequestLoan && order.limit_price > limit_price)
            || (order_type == Action::RequestBorrow && order.limit_price < limit_price)
        {
            break;
        }

        console_log!("limit price check okay");

        // Current behaviour of the debt markets is to abort the transaction on self-match,
        // so we indicate as much if we antipate a self-match. This check must be maintained
        // consitently with the actual orderbook configuration.

        if order.owned_by(&taker) {
            return Err(FixedTermWasmError::SelfMatch.into());
        }

        // Determine the filled amount, making sure to respect both base and quote
        // quantities available on the order.

        console_log!("* check unfilled_quote={} nonzero", unfilled_quote);
        console_log!(
            "* quote_to_base() gives: {}",
            quote_to_base(
                std::cmp::min(unfilled_quote, order.quote_size),
                order.limit_price,
            )
        );

        let bfill = std::cmp::min(
            quote_to_base(
                std::cmp::min(unfilled_quote, order.quote_size),
                order.limit_price,
            ),
            order.base_size,
        );
        let qfill = base_to_quote(bfill, order.limit_price);

        console_log!("Order filled {} quote and {} base", qfill, bfill);

        filled_quote += qfill;
        filled_base += bfill;
        unfilled_quote = unfilled_quote.saturating_sub(qfill);
        matches += 1;

        console_log!("Cumulatively filled {} quote and {} base", filled_quote, filled_base);
        console_log!("{} quote remaining", unfilled_quote);
    }

    console_log!(
        "filled_quote: {}, filled_base: {}",
        filled_quote, filled_base
    );
    console_log!("vwap_f64: {}", filled_quote as f64 / filled_base as f64);
    console_log!("vwap_fp32: {}", f64_to_fp32(filled_quote as f64 / filled_base as f64));

    let vwap = if filled_quote == 0 || filled_base == 0 {
        0
    } else {
        f64_to_fp32(filled_quote as f64 / filled_base as f64)
    };

    Ok(EstimatedOrderOutcome {
        vwap,
        filled_quote,
        filled_base,
        unfilled_quote,
        matches,
    })
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

#[test]
fn test_calculate_implied_price() {
    let base_sz = 100;
    let quote_sz = 50;

    assert_eq!(
        calculate_implied_price(base_sz, quote_sz),
        f64_to_fp32(0.5),
    );
}
