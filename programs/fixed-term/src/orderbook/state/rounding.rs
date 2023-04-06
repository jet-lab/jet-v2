use std::convert::TryInto;

use anchor_lang::{error, Result};

use crate::FixedTermErrorCode;

/// Describes the action being performed, in order to determine the rounding direction
pub enum RoundingAction {
    /// A borrow order has been posted to the orderbook
    PostBorrow,
    /// A borrow order has been filled
    FillBorrow,
    /// A borrow order has been cancelled
    CancelBorrow,
    /// A lend order has been posted to the orderbook
    PostLend,
    /// A lend order has been filled
    FillLend,
    /// A lend order has been cancelled
    CancelLend,
}

/// Should a lamport be added or removed when casting
pub enum RoundingDirection {
    Up,
    Down,
}

impl RoundingAction {
    pub fn direction(&self) -> RoundingDirection {
        use RoundingAction::*;
        use RoundingDirection::*;

        match self {
            PostBorrow => Down,
            FillBorrow => Down,
            CancelBorrow => Down,
            PostLend => Down,
            FillLend => Down,
            CancelLend => Down,
        }
    }
}

/// FIXME: Rounding
pub fn quote_from_base(base: u64, price: u64, rounding: RoundingDirection) -> Result<u64> {
    match rounding {
        RoundingDirection::Up => fp32_mul_ceil(base, price),
        RoundingDirection::Down => fp32_mul_floor(base, price),
    }
    .ok_or_else(|| error!(FixedTermErrorCode::FixedPointMath))
}

/// Multiply a decimal [u64] with a fixed point 32 number
/// a is fp0, b is fp32 and result is a*b fp0
pub fn fp32_mul_floor(a: u64, b_fp32: u64) -> Option<u64> {
    (a as u128)
        .checked_mul(b_fp32 as u128)
        .and_then(|x| (x >> 32).try_into().ok())
}

/// Multiply a decimal [u64] with a fixed point 32 number
/// a is fp0, b is fp32 and result is a*b fp0
pub fn fp32_mul_ceil(a: u64, b_fp32: u64) -> Option<u64> {
    (a as u128)
        .checked_mul(b_fp32 as u128)
        .and_then(fp32_ceil_util)
        .and_then(|x| (x >> 32).try_into().ok())
}

/// a is fp0, b is fp32 and result is a/b fp0
pub fn fp32_div(a: u64, b_fp32: u64) -> Option<u64> {
    ((a as u128) << 32)
        .checked_div(b_fp32 as u128)
        .and_then(|x| x.try_into().ok())
}

#[inline(always)]
fn fp32_ceil_util(x_fp32: u128) -> Option<u128> {
    let add_one = (!(x_fp32 as u32)).wrapping_add(1) as u128;
    x_fp32.checked_add(add_one)
}
