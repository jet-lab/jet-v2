const ONE_IN_BPS: u64 = 100_00;
const LARGEST_PRECISE_DISBURSEMENT: u64 = u64::MAX / ONE_IN_BPS;

/// based on some requested amount, determines the size of a borrow order that
/// adds an origination fee.
pub fn borrow_order_quote(requested_quote: u64, origination_fee_rate_bps: u64) -> u64 {
    if requested_quote == 0 {
        0
    } else if origination_fee_rate_bps == 0 {
        requested_quote
    } else if requested_quote < u64::MAX / origination_fee_rate_bps {
        1 + requested_quote + requested_quote * origination_fee_rate_bps / ONE_IN_BPS
    } else {
        1 + requested_quote + (requested_quote / ONE_IN_BPS) * origination_fee_rate_bps
    }
}

/// Based on some quote amount that was filled from an order that includes a
/// fee, returns the size of the loan disbursement after the fee is deducted.
pub fn disburse(filled_quote: u64, origination_fee_rate_bps: u64) -> u64 {
    if filled_quote == 0 {
        0
    } else if origination_fee_rate_bps == 0 {
        filled_quote
    } else if filled_quote < LARGEST_PRECISE_DISBURSEMENT {
        ONE_IN_BPS * filled_quote / (ONE_IN_BPS + origination_fee_rate_bps)
    } else {
        ONE_IN_BPS * (filled_quote / (ONE_IN_BPS + origination_fee_rate_bps))
    }
    /* Derivation
    ---- assume
    disburse = requested                 // requirement
    order = requested + requested * fee  // borrow_order_quote
    fee = fee_bps / ONE_IN_BPS

    ---- solve for disburse
    order = disburse + disburse * fee
    order = disburse * (1 + fee)
    disburse = order / (1 + fee)
    disburse = order / (1 + fee_bps/ONE_IN_BPS)

    ---- rearrange to minimize integer overflow/underflow
    disburse = order / ((ONE_IN_BPS + fee_bps)/ONE_IN_BPS)
    disburse = ONE_IN_BPS * order / (ONE_IN_BPS + fee_bps)
    */
}

fn smallest_known_order_that_is_too_large(origination_fee_rate_bps: u64) -> u64 {
    ONE_IN_BPS * (u64::MAX / (ONE_IN_BPS + origination_fee_rate_bps) + 1)
}

fn largest_known_order_that_will_work(origination_fee_rate_bps: u64) -> u64 {
    ONE_IN_BPS * (u64::MAX / (ONE_IN_BPS + origination_fee_rate_bps)) - 1
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn disburse_0_as_0() {
        for rate in 0..ONE_IN_BPS {
            assert_eq!(0, borrow_order_quote(0, rate));
            assert_eq!(0, disburse(0, rate));
        }
    }

    #[test]
    fn disburse_as_requested() {
        let u64_sample_set = (0..63)
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
            .collect::<Vec<_>>();
        for rate in 0..ONE_IN_BPS {
            for &i in &u64_sample_set {
                assert_disbursement_matches_request(i, rate)
            }
        }
    }

    #[test]
    fn disburse_largest_allowed_order_as_requested() {
        for rate in 0..ONE_IN_BPS {
            assert_disbursement_matches_request(largest_known_order_that_will_work(rate), rate);
        }
    }

    #[test]
    fn cannot_request_above_than_largest_allowed_order() {
        std::panic::set_hook(Box::new(|_| {}));
        for rate in 1..ONE_IN_BPS {
            for request in [smallest_known_order_that_is_too_large(rate)] {
                std::panic::catch_unwind(|| {
                    assert_disbursement_matches_request(request, rate);
                })
                .expect_err(&format!(
                    "{} - {}",
                    rate,
                    smallest_known_order_that_is_too_large(rate)
                ));
            }
        }
    }

    fn assert_disbursement_matches_request(request: u64, fee: u64) {
        let disburse = disburse(borrow_order_quote(request, fee), fee);
        if request < LARGEST_PRECISE_DISBURSEMENT / 2 && (fee == 0 || request < u64::MAX / fee) {
            if request != disburse {
                panic!(
                    "for fee {} and requested {}, got disbursement {}",
                    fee, request, disburse
                )
            }
        } else {
            if request - disburse > ONE_IN_BPS {
                panic!(
                    "for fee {} and requested {}, got disbursement {}",
                    fee, request, disburse
                )
            }
        }
    }
}
