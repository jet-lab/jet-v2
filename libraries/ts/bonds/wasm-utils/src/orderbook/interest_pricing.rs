#![allow(unused)]
//! this has a bunch of alternative implementations for converting between
//! interest rates and ticket prices. not all are currently in use, but they're
//! kept around to enable easy swapping out as we decide how to show interest to
//! users in the ui
//!
//! definitions:
//! - interest rate: general term for a number that represents growth in value.
//!   may or may not involve compounding. r% interest represents growth from x
//!   to x plus r% of x.
//! - yield: the actual growth in value of an investment, including any
//!   potential effects of compounding.
//! - nominal rate: The rate of return that does not include compounding.
//!   Typically, it has a compounding period that differs from the rate term.
//!   For example, you may have a compounding period of 1 month, and your yield
//!   over that month would be 1%. The yearly nominal rate is calculated by
//!   multiplying the monthly rate of return by 12, which is 12%. Likewise, this
//!   ignores the effects of compounding. To calculate actual yield, you need to
//!   know both the rate term and the compounding period.
//! - continuous nominal rate: Nominal rate with an instantaneous compounding
//!   period. Yield is calculated by compounding this rate continuously.
//! - price: the price of a ticket. price = 1 / (1 + yield)
//! - APY - Annual Percent Yield: yield that would be realized after one year.
//! - APR - Annual Percent Rate: continuous nominal rate with a yearly term.

use std::f64::consts::E;

use jet_proto_math::{
    fixed_point::{Fp32, FP32_ONE},
    number::Number,
};

const SECONDS_PER_YEAR: u64 = 31_536_000;

pub type PricerImpl = AprPricer;

pub trait InterestPricer {
    fn yearly_interest_bps_to_fp32_price(interest_bps: u64, tenor_seconds: u64) -> u64 {
        let px = f64_to_fp32(
            1.0 / (1.0
                + Self::interest_rate_to_yield(
                    bps_to_f64(interest_bps),
                    SECONDS_PER_YEAR as f64,
                    tenor_seconds as f64,
                )),
        );
        assert!(px > 0);
        px
    }
    fn price_fp32_to_bps_yearly_interest(price_fp32: u64, tenor_seconds: u64) -> u64 {
        f64_to_bps(Self::yield_to_interest_rate(
            1.0 / fp32_to_f64(price_fp32) - 1.0,
            tenor_seconds as f64,
            SECONDS_PER_YEAR as f64,
        ))
    }
    fn interest_rate_to_yield(interest_rate: f64, interest_term: f64, yield_term: f64) -> f64;
    fn yield_to_interest_rate(price: f64, yield_term: f64, interest_term: f64) -> f64;
}

pub struct NominalRatePricer;
impl InterestPricer for NominalRatePricer {
    fn interest_rate_to_yield(interest_rate: f64, interest_term: f64, yield_term: f64) -> f64 {
        nominal_interest_rate_conversion(interest_rate, interest_term, yield_term)
    }

    fn yield_to_interest_rate(price: f64, yield_term: f64, interest_term: f64) -> f64 {
        nominal_interest_rate_conversion(price, yield_term, interest_term)
    }
}

/// yearly interest = yearly rate that is compounded continuously for the tenor duration to receive the price
pub struct AprPricer;
impl InterestPricer for AprPricer {
    fn interest_rate_to_yield(interest_rate: f64, interest_term: f64, yield_term: f64) -> f64 {
        continuous_nominal_rate_to_yield(interest_rate, interest_term, yield_term)
    }

    fn yield_to_interest_rate(price: f64, yield_term: f64, interest_term: f64) -> f64 {
        yield_to_continuous_nominal_rate(price, yield_term, interest_term)
    }
}

/// for tenor < 1y: yearly interest = annualized yield that would be received from compounding each tenor over 1y
/// for tenor > 1y: yearly interest = annualized yield that would need to be compounded to ultimately receive the price of the tenor
pub struct ApyPricer;
impl InterestPricer for ApyPricer {
    fn interest_rate_to_yield(interest_rate: f64, interest_term: f64, yield_term: f64) -> f64 {
        yield_to_yield(interest_rate, interest_term, yield_term)
    }

    fn yield_to_interest_rate(price: f64, yield_term: f64, interest_term: f64) -> f64 {
        yield_to_yield(price, yield_term, interest_term)
    }
}

pub fn f64_to_fp32(f: f64) -> u64 {
    let shifted = f * (1u64 << 32) as f64;
    assert!(shifted <= u64::MAX as f64);
    assert!(shifted >= 0.0);
    shifted.round() as u64
}

pub fn fp32_to_f64(fp: u64) -> f64 {
    (fp as f64) / (1u64 << 32) as f64
}

pub fn f64_to_bps(f: f64) -> u64 {
    let bps = f * 10_000.0;
    assert!(bps <= u64::MAX as f64);
    assert!(bps >= 0.0);
    bps.round() as u64
}

pub fn bps_to_f64(bps: u64) -> f64 {
    bps as f64 / 10_000.0
}

/// rate is continuously compounded over some rate_term
/// yield is the total interest that would occur over the yield term with continuous compounding
pub fn continuous_nominal_rate_to_yield(rate: f64, rate_term: f64, yield_term: f64) -> f64 {
    E.powf(rate * yield_term / rate_term) - 1f64
}

/// rate is continuously compounded over some rate_term
/// yield is the total interest that would occur over the yield term with continuous compounding
pub fn yield_to_continuous_nominal_rate(yld: f64, yield_term: f64, rate_term: f64) -> f64 {
    (yld + 1.0).ln() * rate_term / yield_term
}

/// compounds over the smaller periods to get to the larger period
pub fn yield_to_yield(input: f64, input_term: f64, output_term: f64) -> f64 {
    (1f64 + input).powf(output_term / input_term) - 1f64
}

/// Converts one interest rate to another by scaling it linearly
pub fn nominal_interest_rate_conversion(input_rate: f64, input_term: f64, output_term: f64) -> f64 {
    input_rate * output_term / input_term
}

pub fn linear_rate_to_price_number(interest_rate: u64, tenor: u64) -> u64 {
    let year_proportion = Number::from(tenor) / SECONDS_PER_YEAR;
    let rate = Number::from(interest_rate) / 10_000;
    let price = (Number::ONE / (Number::ONE + rate * year_proportion)) * FP32_ONE;
    Fp32::wrap_u128(price.as_u128(0)).downcast_u64().unwrap()
}

// TODO:
// (rate, tenor) -> price
// price = 1 / (1 + rate * tenor)
// price * (1 + rate * tenor) = 1
// tenor: as fraction of the period. Period is always annual
// let price = x;
// assert(price == rate_to_price(price_to_rate(price, tenor), tenor));
// rate  = (1 - price) / tenor * price
pub fn price_to_linear_rate_number(price: u64, tenor: u64) -> u64 {
    let year_proportion = Number::from(tenor) / SECONDS_PER_YEAR;
    let price = Number::from(price) / FP32_ONE; // convert to decimal representation
    let rate = (Number::ONE - price) / year_proportion * price;
    (rate * 10_000).as_u64(0)
}

#[cfg(test)]
mod test {
    use rand_chacha::rand_core::{RngCore, SeedableRng};

    use super::*;

    /// any price that would cause negative interest cannot be represented
    /// correctly by u64 and it makes no sense as a loan.
    #[test]
    #[should_panic]
    #[allow(arithmetic_overflow)] // ensures that we're not relying on test-specific behavior
    fn price_cannot_be_greater_than_one() {
        // 3<<31 is 1.5 in fp32
        PricerImpl::price_fp32_to_bps_yearly_interest(3 << 31, SECONDS_PER_YEAR);
    }

    /// price of zero is nonsense. this means you pay back interest on a loan
    /// that had no principal. in other words it's an infinite interest rate,
    /// so you can't actually represent it with a u64.
    #[test]
    #[should_panic]
    #[allow(arithmetic_overflow)] // ensures that we're not relying on test-specific behavior
    fn price_cannot_be_zero() {
        PricerImpl::price_fp32_to_bps_yearly_interest(0, SECONDS_PER_YEAR);
    }

    /// since this is a test then an overflow would fail, unlike in production
    /// code. this just makes sure that the lowest price greater than 1 can
    /// still result in an interest rate that isn't going to overflow u64
    #[test]
    fn price_may_be_small() {
        PricerImpl::price_fp32_to_bps_yearly_interest(1, SECONDS_PER_YEAR);
    }

    #[test]
    fn price_of_one_is_zero_interest() {
        assert_eq!(
            0,
            PricerImpl::price_fp32_to_bps_yearly_interest(1 << 32, SECONDS_PER_YEAR)
        );
        assert_eq!(
            1 << 32,
            PricerImpl::yearly_interest_bps_to_fp32_price(0, SECONDS_PER_YEAR)
        );
    }

    #[test]
    #[should_panic]
    #[allow(arithmetic_overflow)] // ensures that we're not relying on test-specific behavior
    fn rate_should_be_capped_to_prevent_nonsensical_price_of_zero() {
        PricerImpl::yearly_interest_bps_to_fp32_price(1 << 18, SECONDS_PER_YEAR);
    }

    #[test]
    fn conversions() {
        generic_conversions::<PricerImpl>()
    }

    #[test]
    fn conversions_linear() {
        generic_conversions::<NominalRatePricer>()
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
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(10);
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
        use super::*;

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
        /// equivalent yield. Nominal pricing without a standard compounding
        /// term suggests they have a different yield, which is not helpful.
        #[test]
        fn equal_profitability() {
            let monthly_price = f64_to_fp32(1.0 / 1.0079741404289037);
            let yearly_price = f64_to_fp32(1.0 / 1.1);

            assert_eq!(
                ApyPricer::price_fp32_to_bps_yearly_interest(monthly_price, SECONDS_PER_YEAR / 12),
                ApyPricer::price_fp32_to_bps_yearly_interest(yearly_price, SECONDS_PER_YEAR)
            );
            assert_eq!(
                AprPricer::price_fp32_to_bps_yearly_interest(monthly_price, SECONDS_PER_YEAR / 12),
                AprPricer::price_fp32_to_bps_yearly_interest(yearly_price, SECONDS_PER_YEAR)
            );
            // Nominal pricing with differing compounding terms (based on the
            // tenor) suggest that the monthly has lower interest, which would
            // imply that you should invest in the yearly unless you need the
            // liquidity of the shorter term loan. This is contrary to the
            // conclusion described in the rustdoc. This form of nominal pricing
            // is not effective at comparing different tenors. A common
            // compounding term is required to make interest rates comparable
            // across different tenors, which means the tenor itself cannot be
            // the compounding term.  A continuous compounding term seems to be
            // the most universal choice.
            assert!(
                NominalRatePricer::price_fp32_to_bps_yearly_interest(
                    monthly_price,
                    SECONDS_PER_YEAR / 12
                ) < NominalRatePricer::price_fp32_to_bps_yearly_interest(
                    yearly_price,
                    SECONDS_PER_YEAR
                )
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
            actual_price as f64 / (1u64 << 32) as f64,
        );
    }

    #[test]
    fn happy_path() {
        roughly_eq(
            0.105_170_918,
            continuous_nominal_rate_to_yield(0.1, 1.0, 1.0),
        );
        roughly_eq(0.126_825_030_131_969_72, yield_to_yield(0.01, 1.0, 12.0));
    }

    fn roughly_eq(x: f64, y: f64) {
        let diff = (x - y).abs();
        if diff > 0.000_000_001 * x || diff > 0.000_000_001 * y {
            panic!("\nnot roughly equal:\n  {x}\n  {y}\n")
        }
    }
}
