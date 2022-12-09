const FEE_UNIT: u64 = 1_000_000;

/// based on some requested amount, determines the size of a borrow order that
/// adds an origination fee.
/// - origination_fee: scaled by FEE_RATIO
pub fn borrow_order_qty(requested: u64, origination_fee: u64) -> u64 {
    if requested == 0 {
        0
    } else if origination_fee == 0 {
        requested
    } else {
        let requested = requested as u128;
        let quote = 1 + requested + (requested * origination_fee as u128) / FEE_UNIT as u128;
        if quote >= u64::MAX as u128 {
            u64::MAX
        } else {
            quote as u64
        }
    }
}

/// Based on some quote amount that was filled from an order that includes a
/// fee, returns the size of the loan disbursement after the fee is deducted.
pub fn loan_to_disburse(filled_quote: u64, origination_fee: u64) -> u64 {
    if filled_quote == 0 {
        0
    } else if origination_fee == 0 {
        filled_quote
    } else if filled_quote < u64::MAX / FEE_UNIT {
        FEE_UNIT * filled_quote / (FEE_UNIT + origination_fee)
    } else {
        // FEE_UNIT * (filled_quote / (FEE_UNIT + origination_fee))
        (FEE_UNIT as u128 * filled_quote as u128 / (FEE_UNIT as u128 + origination_fee as u128))
            as u64
    }
    /* Derivation
    ---- assume
    disburse = requested                          // requirement
    order = requested + requested * fee/FEE_UNIT  // borrow_order_qty

    ---- solve for disburse
    order = disburse + disburse * fee/FEE_UNIT
    order = disburse * (1 + fee/FEE_UNIT)
    disburse = order / (1 + fee/FEE_UNIT)
    disburse = order / (1 + fee/FEE_UNIT)

    ---- rearrange to minimize integer overflow/underflow
    disburse = order / ((FEE_UNIT + fee)/FEE_UNIT)
    disburse = FEE_UNIT * order / (FEE_UNIT + fee)
    */
}

#[cfg(test)]
mod test {
    use super::*;

    /// should be much larger than anyone would ever imagine charging for an origination fee
    const LARGEST_CONVEIVABLE_RATE: u64 = (0.001 * FEE_UNIT as f64) as u64;

    /// returns the largest known quantity that will *not* result in an origination
    /// fee that sums with the order size to be greater than u64::MAX. larger
    /// numbers may overflow. there are likely larger numbers that can be borrowed
    /// without overflowing, but they are definitely smaller than
    /// smallest_amount_known_to_be_unborrowable()
    fn largest_amount_known_to_be_borrowable(origination_fee: u64) -> u64 {
        if origination_fee == 0 {
            u64::MAX
        } else {
            (FEE_UNIT as u128 * (u64::MAX - 1) as u128
                / (FEE_UNIT as u128 + origination_fee as u128)) as u64
        }
    }

    /// returns a quantity that will definitely result in an origination fee
    /// that sums with the order size to be greater than u64::MAX. there are
    /// likely smaller numbers with the same problem, but they are definitely
    /// larger than largest_amount_known_to_be_borrowable()
    fn smallest_amount_known_to_be_unborrowable(origination_fee: u64) -> Option<u64> {
        if origination_fee == 0 {
            None
        } else {
            Some(
                (FEE_UNIT as u128 * (u64::MAX - 1) as u128
                    / (FEE_UNIT as u128 + origination_fee as u128)) as u64
                    + 2,
            )
        }
    }

    /// ensures that the boundaries we're using in other logic define an actual
    /// boundary. the boundary may be fuzzy, containing some ambiguous values,
    /// but at least it must be accurate as stated.
    #[test]
    fn borrowability_is_well_defined() {
        for fee in 0..LARGEST_CONVEIVABLE_RATE {
            let safe = largest_amount_known_to_be_borrowable(fee) as u128;
            if fee > 0 {
                if 1 + safe + (fee as u128 * safe) / FEE_UNIT as u128 > u64::MAX as u128 {
                    panic!("quantity not borrowable {} - {}", safe, fee);
                }
                let bad = smallest_amount_known_to_be_unborrowable(fee).unwrap() as u128;
                if 1 + bad + (fee as u128 * bad) / FEE_UNIT as u128 <= u64::MAX as u128 {
                    panic!("quantity is borrowable {} - {}", bad, fee);
                }
            } else {
                assert_eq!(u64::MAX as u128, safe);
                assert_eq!(None, smallest_amount_known_to_be_unborrowable(fee));
            }
        }
    }

    #[test]
    fn disburse_0_as_0() {
        for rate in 0..LARGEST_CONVEIVABLE_RATE {
            assert_eq!(0, borrow_order_qty(0, rate));
            assert_eq!(0, loan_to_disburse(0, rate));
        }
    }

    #[test]
    fn disburse_as_requested() {
        let sample = u64_lower_sample_set();
        for rate in 0..LARGEST_CONVEIVABLE_RATE {
            let max = largest_amount_known_to_be_borrowable(rate);
            for request in sample.iter().flat_map(|&n| [n, max - n]) {
                for number_of_fills in 1..10 {
                    assert_disbursement_matches_request(request, rate, number_of_fills)
                }
            }
        }
    }

    /// samples the u64 domain starting at 0 without reaching the top
    /// powers of two, primes, and polynomial
    fn u64_lower_sample_set() -> Vec<u64> {
        (0..63)
            .into_iter()
            .flat_map(|x| {
                let p = 2u64.pow(x);
                p - 1..p + 1
            })
            .chain((0u64..41).into_iter().map(|x| x.pow(12)))
            .collect()
    }

    /// not a very realistic scenario but establishes reasonable boundary conditions
    #[test]
    fn orders_that_could_overflow_are_pushed_to_u64_max() {
        let sample = u64_lower_sample_set();
        for rate in 1..LARGEST_CONVEIVABLE_RATE {
            let bad = smallest_amount_known_to_be_unborrowable(rate).unwrap();
            let ok = largest_amount_known_to_be_borrowable(rate);
            assert!(bad > ok);
            for request in ok..bad {
                // numbers around the edge might not overflow but they should be very close
                assert!(u64::MAX - borrow_order_qty(request, rate) <= 2);
            }
            for request in sample.iter().filter_map(|n| n.checked_add(bad)) {
                assert_eq!(u64::MAX, borrow_order_qty(request, rate));
            }
            assert_eq!(u64::MAX, borrow_order_qty(u64::MAX, rate));
        }
    }

    /// not a very realistic scenario but establishes reasonable boundary conditions
    #[test]
    fn large_disbursements_from_overflowing_orders_must_cover_full_fee_and_a_reasonable_amount_of_the_request(
    ) {
        for rate in 1..LARGEST_CONVEIVABLE_RATE {
            let disburse = loan_to_disburse(u64::MAX, rate);
            let realized_fee = u64::MAX - disburse;
            let expected_fee = (disburse / FEE_UNIT) * rate;
            let a_smaller_order_that_wouldnt_overflow = largest_amount_known_to_be_borrowable(rate);
            assert!(disburse >= a_smaller_order_that_wouldnt_overflow);
            assert!(expected_fee <= realized_fee); // fee should be covered
        }
    }

    fn assert_disbursement_matches_request(request: u64, fee: u64, number_of_fills: u64) {
        let mut total_disbursed = 0;
        let mut quote = borrow_order_qty(request, fee);
        for _ in 0..(number_of_fills - 1) {
            let fill = if quote > 1 { quote / 2 } else { quote };
            quote -= fill;
            let disburse = loan_to_disburse(fill, fee);
            total_disbursed += disburse;
        }
        total_disbursed += loan_to_disburse(quote, fee);

        if total_disbursed > request || request - total_disbursed > number_of_fills - 1 {
            panic!("
                for fee {fee} and effective request {request} with quote {quote} in {number_of_fills} fills, 
                was disbursed {total_disbursed}, with a discrepancy of {}, but expected max discrepancy of {}",
                request as f64 - total_disbursed as f64, number_of_fills - 1
                )
        }
    }
}
