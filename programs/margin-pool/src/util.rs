// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright (C) 2022 JET PROTOCOL HOLDINGS, LLC.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use anchor_lang::{prelude::AccountInfo, solana_program::clock::UnixTimestamp};
use jet_proto_math::Number;

pub const SECONDS_PER_HOUR: UnixTimestamp = 3600;
pub const SECONDS_PER_2H: UnixTimestamp = SECONDS_PER_HOUR * 2;
pub const SECONDS_PER_12H: UnixTimestamp = SECONDS_PER_HOUR * 12;
pub const SECONDS_PER_DAY: UnixTimestamp = SECONDS_PER_HOUR * 24;
pub const SECONDS_PER_WEEK: UnixTimestamp = SECONDS_PER_DAY * 7;
pub const SECONDS_PER_YEAR: UnixTimestamp = 31_536_000;
pub const MAX_ACCRUAL_SECONDS: UnixTimestamp = SECONDS_PER_WEEK;

static_assertions::const_assert_eq!(SECONDS_PER_HOUR, 60 * 60);
static_assertions::const_assert_eq!(SECONDS_PER_2H, 60 * 60 * 2);
static_assertions::const_assert_eq!(SECONDS_PER_12H, 60 * 60 * 12);
static_assertions::const_assert_eq!(SECONDS_PER_DAY, 60 * 60 * 24);
static_assertions::const_assert_eq!(SECONDS_PER_WEEK, 60 * 60 * 24 * 7);
static_assertions::const_assert_eq!(SECONDS_PER_YEAR, 60 * 60 * 24 * 365);

/// Computes the effective applicable interest rate assuming continuous
/// compounding for the given number of slots.
///
/// Uses an approximation calibrated for accuracy to twenty decimals places,
/// though the current configuration of Number does not support that.
pub fn compound_interest(rate: Number, seconds: UnixTimestamp) -> Number {
    // The two panics below are implementation details, chosen to facilitate convenient
    // implementation of compounding. They can be relaxed with a bit of additional work.
    // The "seconds" guards are chosen to guarantee accuracy under the assumption that
    // the rate is not more than one.

    if rate > Number::ONE * 2 {
        panic!("Not implemented; interest rate too large for compound_interest()");
    }

    let terms = match seconds {
        _ if seconds <= SECONDS_PER_2H => 5,
        _ if seconds <= SECONDS_PER_12H => 6,
        _ if seconds <= SECONDS_PER_DAY => 7,
        _ if seconds <= SECONDS_PER_WEEK => 10,
        _ => panic!("Not implemented; too many seconds in compound_interest()"),
    };

    let x = rate * seconds / SECONDS_PER_YEAR;

    jet_proto_math::expm1_approx(x, terms)
}

/// Linear interpolation between (x0, y0) and (x1, y1).
pub fn interpolate(x: Number, x0: Number, x1: Number, y0: Number, y1: Number) -> Number {
    assert!(x >= x0);
    assert!(x <= x1);

    y0 + ((x - x0) * (y1 - y0)) / (x1 - x0)
}

/// Get a token mint's supply
pub fn supply(account: &AccountInfo) -> anchor_lang::Result<u64> {
    let bytes = account.try_borrow_data()?;
    let mut amount_bytes = [0u8; 8];
    amount_bytes.copy_from_slice(&bytes[36..44]);
    Ok(u64::from_le_bytes(amount_bytes))
}
