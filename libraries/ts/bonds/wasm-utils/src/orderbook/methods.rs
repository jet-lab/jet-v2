use std::f64::consts::E;

use jet_proto_math::{
    fixed_point::{Fp32, FP32_ONE},
    number::Number,
};
use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::*;

use super::{critbit::Slab, types::Order};
use interest_pricing::InterestPricer;

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

/// Given a price and bond duration, calculates an interest rate
///
/// price: underlying per bond ticket: fixed point 32 (left shifted 32 bits to get fractional precision)
/// tenor: seconds
/// return: interest rate in basis points
#[wasm_bindgen]
pub fn price_to_rate(price: u64, tenor: u64) -> u64 {
    interest_pricing::PricerImpl::price_fp32_to_bps_yearly_interest(price, tenor)
}

/// Given an interest rate and bond duration, calculates a price
///
/// interest_rate: basis points
/// tenor: seconds
/// return: price: underlying per bond ticket: fixed point 32 (left shifted 32 bits to get fractional precision)
#[wasm_bindgen]
pub fn rate_to_price(interest_rate: u64, tenor: u64) -> u64 {
    interest_pricing::PricerImpl::yearly_interest_bps_to_fp32_price(interest_rate, tenor)
}

/// this has a bunch of alternative implementations for converting between
/// interest rates and ticket prices.
/// not all are currently in use, but they're kept around to enable easy
/// swapping out as we decide how to show interest to users in the ui
#[allow(unused)]
mod interest_pricing {
    use super::*;

    pub type PricerImpl = AprPricer;

    pub trait InterestPricer {
        fn yearly_interest_bps_to_fp32_price(interest_bps: u64, tenor_seconds: u64) -> u64 {
            f64_to_fp32(1.0 / (1.0 + Self::interest_to_single_term_yield(
                bps_to_f64(interest_bps),
                SECONDS_PER_YEAR as f64,
                tenor_seconds as f64,
            )))
        }
        fn price_fp32_to_bps_yearly_interest(price_fp32: u64, tenor_seconds: u64) -> u64 {
            f64_to_bps(Self::single_term_yield_to_interest(
                1.0 / fp32_to_f64(price_fp32) - 1.0,
                tenor_seconds as f64,
                SECONDS_PER_YEAR as f64,
            ))
        }
        /// based on the number representing "interest", return the proportion of growth over the term of one loan
        fn interest_to_single_term_yield(interest: f64, interest_term: f64, price_term: f64) -> f64;
        /// based on the proportion of growth over the term of one loan, return the number representing "interest"
        fn single_term_yield_to_interest(price: f64, price_term: f64, interest_term: f64) -> f64;
    }

    pub struct LinearPricer;
    impl InterestPricer for LinearPricer {
        fn interest_to_single_term_yield(interest_rate: f64, interest_term: f64, price_term: f64) -> f64 {
            linear_uncompounded_interest_conversion(interest_rate, interest_term, price_term)
        }

        fn single_term_yield_to_interest(price: f64, price_term: f64, interest_term: f64) -> f64 {
            linear_uncompounded_interest_conversion(price, price_term, interest_term)
        }
    }

    /// yearly interest = yearly rate that is compounded continuously for the tenor duration to receive the price
    pub struct AprPricer;
    impl InterestPricer for AprPricer {
        fn interest_to_single_term_yield(interest_rate: f64, interest_term: f64, price_term: f64) -> f64 {
            rate_to_yield(interest_rate, interest_term, price_term)
        }

        fn single_term_yield_to_interest(price: f64, price_term: f64, interest_term: f64) -> f64 {
            yield_to_rate(price, price_term, interest_term)
        }
    }

    /// for tenor < 1y: yearly interest = annualized yield that would be received from compounding each tenor over 1y
    /// for tenor > 1y: yearly interest = annualized yield that would need to be compounded to ultimately receive the price of the tenor
    pub struct ApyPricer;
    impl InterestPricer for ApyPricer {
        fn interest_to_single_term_yield(interest_rate: f64, interest_term: f64, price_term: f64) -> f64 {
            yield_to_yield(interest_rate, interest_term, price_term)
        }

        fn single_term_yield_to_interest(price: f64, price_term: f64, interest_term: f64) -> f64 {
            yield_to_yield(price, price_term, interest_term)
        }
    }

    pub fn f64_to_fp32(f: f64) -> u64 {
        let shifted = f * (2u64 << 32) as f64;
        assert!(shifted < u64::MAX as f64);
        shifted.round() as u64
    }

    pub fn fp32_to_f64(fp: u64) -> f64 {
        (fp as f64) / (2u64 << 32) as f64
    }

    pub fn f64_to_bps(f: f64) -> u64 {
        (f * 10_000.0).round() as u64
    }

    pub fn bps_to_f64(bps: u64) -> f64 {
        bps as f64 / 10_000.0
    }

    /// rate is continuously compounded over some rate_term
    /// yield is the total interest that would occur over the yield term with continuous compounding
    pub fn rate_to_yield(rate: f64, rate_term: f64, yield_term: f64) -> f64 {
        E.powf(rate * yield_term / rate_term) - 1f64
    }

    /// rate is continuously compounded over some rate_term
    /// yield is the total interest that would occur over the yield term with continuous compounding
    pub fn yield_to_rate(yld: f64, yield_term: f64, rate_term: f64) -> f64 {
        (yld + 1.0).ln() * rate_term / yield_term
    }

    /// compounds over the smaller periods to get to the larger period
    pub fn yield_to_yield(input: f64, input_term: f64, output_term: f64) -> f64 {
        (1f64 + input).powf(output_term / input_term) - 1f64
    }

    pub fn linear_uncompounded_interest_conversion(
        input: f64,
        input_term: f64,
        output_term: f64,
    ) -> f64 {
        input * output_term / input_term
    }

    pub fn linear_rate_to_price_number(interest_rate: u64, tenor: u64) -> u64 {
        let year_proportion = Number::from(tenor) / SECONDS_PER_YEAR;
        let rate = Number::from(interest_rate) / 10_000;
        let price = (Number::ONE / (Number::ONE + rate * year_proportion)) * FP32_ONE;
        Fp32::wrap_u128(price.as_u128(0)).downcast_u64().unwrap()
    }

    // rate  = (1 - price) / tenor * price
    pub fn price_to_linear_rate_number(price: u64, tenor: u64) -> u64 {
        let year_proportion = Number::from(tenor) / SECONDS_PER_YEAR;
        let price = Number::from(price) / FP32_ONE; // convert to decimal representation
        let rate = (Number::ONE - price) / year_proportion * price;
        (rate * 10_000).as_u64(0)
    }
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
    use crate::orderbook::methods::interest_pricing::*;

    use super::SECONDS_PER_YEAR;

    #[test]
    fn conversions() {
        generic_conversions::<PricerImpl>()
    }

    #[test]
    fn conversions_linear() {
        generic_conversions::<LinearPricer>()
    }

    #[test]
    fn conversions_apr() {
        generic_conversions::<AprPricer>()
    }

    #[test]
    fn conversions_apy() {
        generic_conversions::<ApyPricer>()
    }

    fn generic_conversions<P: InterestPricer>() {
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
            assert_eq!(
                rate,
                P::price_fp32_to_bps_yearly_interest(
                    P::yearly_interest_bps_to_fp32_price(rate, tenor),
                    tenor
                )
            )
        }
    }

    #[test]
    fn apy() {
        let apy_bps = 1000;
        assert_price_generates_expected_yield::<ApyPricer>(
            apy_bps,
            SECONDS_PER_YEAR / 12,
            0.007974140428903741,
        );
        assert_price_generates_expected_yield::<ApyPricer>(apy_bps, SECONDS_PER_YEAR, 0.1);
        assert_price_generates_expected_yield::<ApyPricer>(apy_bps, 2 * SECONDS_PER_YEAR, 0.21);
    }

    #[test]
    fn apr() {
        let apr_bps = 1000;
        assert_price_generates_expected_yield::<AprPricer>(
            apr_bps,
            SECONDS_PER_YEAR / 12,
            0.008368152207446989,
        );
        assert_price_generates_expected_yield::<AprPricer>(
            apr_bps,
            SECONDS_PER_YEAR,
            0.10517091807564762,
        );
        assert_price_generates_expected_yield::<AprPricer>(
            apr_bps,
            2 * SECONDS_PER_YEAR,
            0.22140275816016983,
        );
    }

    /// Let's say I'm considering investing in fixed term lending. There are a
    /// handful of tenors I am considering, and the first thing I need to do is
    /// determine the relative profitability of each.
    ///
    /// If I'm comparing a short tenor to a long tenor, I have no choice with
    /// the long tenor over one of its terms: I must keep the full balance
    /// invested. Likewise, to make a meaningful comparison of the longer tenor
    /// to the shorter tenor, I must also assume that the shorter tenor is fully
    /// reinvested after each of its terms until the end of the longer tenor's
    /// first term. Only then can I get a meaningful comparison in yield.
    ///
    /// Of course the shorter tenor is more liquid and that is worth something.
    /// But if we assume some non-zero withdrawal is made between terms of the
    /// shorter tenor, then the relative yields of each tenor or no longer
    /// comparable. And how do we decide how large the withdrawal should be?
    /// It's totally arbitrary.
    ///
    /// The value of the liquidity needs to be independently quantified so it
    /// can be directly compared to the loss in best case performance relative
    /// to a longer tenor. But that cost benefit analysis comes later. First I
    /// just want an objective measure of yield.
    ///
    /// Here are a few scenarios to illustrate why this line of reasoning is the
    /// most intuitive and useful way to compare interest rates. APR and APY
    /// both follow this line of reasoning, whereas the linear approach does
    /// not.
    mod scenarios {
        use crate::orderbook::methods::{interest_pricing::*, SECONDS_PER_YEAR};

        /// Let's say we have a one month tenor and a one year tenor. Somehow,
        /// these loans have been priced such that if you reinvest the monthly's
        /// full balance at the end of each of its terms, at the current rate it
        /// would accumulate the same total yield after one year as the yearly
        /// loan. Obviously, the monthly is the better investment. You lose
        /// nothing in terms of profitability. You only gain a more liquid
        /// position.
        ///
        /// You should only get the yearly if you anticipate that *both* tenors
        /// will have lower rates in the market one month from now. This is an
        /// important consideration, but it is also critical to realize that an
        /// anticipation of this specific price movement is the only reason why
        /// you should buy the yearly. If you think *either* price is more (or
        /// equally) likely to go up than it is to go down, then the monthly is
        /// still the obvious choice.
        /// 
        /// Let's say the yearly grows by 10% after a single year. So if you
        /// lend $100, you'll get $110 at the end. That means its price is
        /// 1/1.1. Likewise, the monthly would need to grow by
        /// 0.00797414042890374107 each month to reach the same total after a
        /// year, because 1.00797414042890374107^12 = 1.1
        /// 
        /// Using either APY or APR, it is clear that the monthly tenors have
        /// equivalent yield. Linear pricing suggests they have a different
        /// yield, which is not helpful.
        #[test]
        fn equal_profitability() {
            let monthly_price = f64_to_fp32(1.0/1.00797414042890374107);
            let yearly_price = f64_to_fp32(1.0/1.1);
            
            assert_eq!(
                ApyPricer::price_fp32_to_bps_yearly_interest(monthly_price, SECONDS_PER_YEAR/12),
                ApyPricer::price_fp32_to_bps_yearly_interest(yearly_price, SECONDS_PER_YEAR)
            );
            assert_eq!(
                AprPricer::price_fp32_to_bps_yearly_interest(monthly_price, SECONDS_PER_YEAR/12),
                AprPricer::price_fp32_to_bps_yearly_interest(yearly_price, SECONDS_PER_YEAR)
            );
            // Linear pricing says that the monthly has lower interest, which
            // would imply that you should invest in the yearly unless you need
            // the liquidity of the shorter term loan. This is contrary to the
            // conclusion described in the rustdoc. Linear pricing is not
            // effective at comparing different tenors.
            assert!(
                LinearPricer::price_fp32_to_bps_yearly_interest(monthly_price, SECONDS_PER_YEAR/12)
                < LinearPricer::price_fp32_to_bps_yearly_interest(yearly_price, SECONDS_PER_YEAR)
            );
        }
    }

    fn assert_price_generates_expected_yield<P: InterestPricer>(
        bps: u64,
        tenor: u64,
        expected_yield: f64,
    ) {
        let actual_price = P::yearly_interest_bps_to_fp32_price(bps, tenor);
        roughly_eq(
            1.0 / (1.0 + expected_yield),
            actual_price as f64 / (2u64 << 32) as f64,
        );
    }

    #[test]
    fn happy_path() {
        roughly_eq(0.105_170_918, rate_to_yield(0.1, 1.0, 1.0));
        roughly_eq(0.126_825_030_131_969_72, yield_to_yield(0.01, 1.0, 12.0));
    }

    fn roughly_eq(x: f64, y: f64) {
        let diff = (x - y).abs();
        if diff > 0.000_000_001 * x || diff > 0.000_000_001 * y {
            panic!("\nnot roughly equal:\n  {x}\n  {y}\n")
        }
    }
}
