const DECIMALS: u64 = 1_000_000;
const LARGEST_PRECISE_DISBURSEMENT: u64 = u64::MAX / DECIMALS;

/// based on some requested amount, determines the size of a borrow order that
/// adds an origination fee.
pub fn borrow_order_quote(requested_quote: u64, origination_fee_rate_dcml: u64) -> u64 {
    if requested_quote == 0 {
        0
    } else if origination_fee_rate_dcml == 0 {
        requested_quote
    } else if requested_quote < u64::MAX / origination_fee_rate_dcml {
        1 + requested_quote + requested_quote * origination_fee_rate_dcml / DECIMALS
    } else {
        1 + requested_quote + (requested_quote / DECIMALS) * origination_fee_rate_dcml
    }
}

/// Based on some quote amount that was filled from an order that includes a
/// fee, returns the size of the loan disbursement after the fee is deducted.
pub fn disburse(filled_quote: u64, origination_fee_rate_dcml: u64) -> u64 {
    if filled_quote == 0 {
        0
    } else if origination_fee_rate_dcml == 0 {
        filled_quote
    } else if filled_quote < LARGEST_PRECISE_DISBURSEMENT {
        DECIMALS * filled_quote / (DECIMALS + origination_fee_rate_dcml)
    } else {
        DECIMALS * (filled_quote / (DECIMALS + origination_fee_rate_dcml))
    }
    /* Derivation
    ---- assume
    disburse = requested                 // requirement
    order = requested + requested * fee  // borrow_order_quote
    fee = fee_dcml / ONE_IN_dcml

    ---- solve for disburse
    order = disburse + disburse * fee
    order = disburse * (1 + fee)
    disburse = order / (1 + fee)
    disburse = order / (1 + fee_dcml/ONE_IN_dcml)

    ---- rearrange to minimize integer overflow/underflow
    disburse = order / ((ONE_IN_dcml + fee_dcml)/ONE_IN_dcml)
    disburse = ONE_IN_dcml * order / (ONE_IN_dcml + fee_dcml)
    */
}

fn smallest_known_order_that_is_too_large(origination_fee_rate_dcml: u64) -> u64 {
    DECIMALS * (u64::MAX / (DECIMALS + origination_fee_rate_dcml) + 1)
}

fn largest_known_order_that_will_work(origination_fee_rate_dcml: u64) -> u64 {
    DECIMALS * (u64::MAX / (DECIMALS + origination_fee_rate_dcml)) - 1
}

#[cfg(test)]
mod test {
    use super::*;

    /// should be much larger than anyone would ever imagine charging for an origination fee
    const LARGEST_CONVEIVABLE_RATE: u64 = (0.001 * DECIMALS as f64) as u64;

    /// when an order is for more than LARGEST_PRECISE_DISBURSEMENT, we expect
    /// to lose lamport-level precision. the best we can do at that size is to
    /// be precise to values of this size.
    const PRECISION_FOR_LARGE_NUMBERS: u64 = DECIMALS;

    #[test]
    fn disburse_0_as_0() {
        for rate in 0..LARGEST_CONVEIVABLE_RATE {
            assert_eq!(0, borrow_order_quote(0, rate));
            assert_eq!(0, disburse(0, rate));
        }
    }

    #[test]
    fn disburse_as_requested() {
        for rate in 0..LARGEST_CONVEIVABLE_RATE {
            for &i in &u64_sample_set() {
                for number_of_fills in 1..10 {
                    assert_disbursement_matches_request(i, rate, number_of_fills)
                }
            }
        }
    }

    fn u64_sample_set() -> Vec<u64> {
        (0..63)
            .into_iter()
            .flat_map(|x| {
                let p = 2u64.pow(x);
                p - 1..p
            })
            .chain((0..63).into_iter().map(|x| 2u64.pow(x) - 1))
            .chain((0..40).into_iter().map(|x| 3u64.pow(x)))
            .chain(
                (0u64..110)
                    .into_iter()
                    .map(|x| 4 * x.pow(9) + 3 * x.pow(4) + 2 * x.pow(2) + 1),
            )
            .collect()
    }

    #[test]
    fn disburse_largest_allowed_order_as_requested() {
        for rate in 0..LARGEST_CONVEIVABLE_RATE {
            assert_disbursement_matches_request(largest_known_order_that_will_work(rate), rate, 1);
        }
    }

    #[test]
    fn cannot_request_above_than_largest_allowed_order() {
        std::panic::set_hook(Box::new(|_| {}));
        for rate in 1..LARGEST_CONVEIVABLE_RATE {
            for request in [smallest_known_order_that_is_too_large(rate)] {
                std::panic::catch_unwind(|| {
                    assert_disbursement_matches_request(request, rate, 1);
                })
                .expect_err(&format!(
                    "{} - {}",
                    rate,
                    smallest_known_order_that_is_too_large(rate)
                ));
            }
        }
    }

    #[test]
    fn precision_is_only_sacrificed_when_negligible() {
        assert!(LARGEST_PRECISE_DISBURSEMENT / PRECISION_FOR_LARGE_NUMBERS > 1_000_000);
    }

    fn assert_disbursement_matches_request(request: u64, fee: u64, number_of_fills: u64) {
        let mut total_disbursed = 0;
        let mut quote = borrow_order_quote(request, fee);
        for _ in 0..(number_of_fills - 1) {
            let fill = if quote > 1 { quote / 2 } else { quote };
            quote -= fill;
            let disburse = disburse(fill, fee);
            total_disbursed += disburse;
        }
        total_disbursed += disburse(quote, fee);

        let max_discrepancy = number_of_fills
            * if request < LARGEST_PRECISE_DISBURSEMENT / 2
                && (fee == 0 || request < u64::MAX / fee)
            {
                1
            } else {
                PRECISION_FOR_LARGE_NUMBERS
            };

        if total_disbursed > request || request - total_disbursed > max_discrepancy {
            panic!(
                    "for fee {} and requested {} in {} fills, was disbursed {}, with a discrepancy of {}, but expected max discrepancy of {}",
                    fee, request, number_of_fills, total_disbursed, request - total_disbursed, max_discrepancy
                )
        }
    }
}
