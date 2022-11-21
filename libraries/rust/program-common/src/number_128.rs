use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use bytemuck::{Pod, Zeroable};

const PRECISION: i32 = 10;
const ONE: i128 = 10_000_000_000;

const POWERS_OF_TEN: &[i128] = &[
    1,
    10,
    100,
    1_000,
    10_000,
    100_000,
    1_000_000,
    10_000_000,
    100_000_000,
    1_000_000_000,
    10_000_000_000,
    100_000_000_000,
    1_000_000_000_000,
    10_000_000_000_000,
    100_000_000_000_000,
    1_000_000_000_000_000,
    10_000_000_000_000_000,
    100_000_000_000_000_000,
    1_000_000_000_000_000_000,
    10_000_000_000_000_000_000,
    100_000_000_000_000_000_000,
    1_000_000_000_000_000_000_000,
    10_000_000_000_000_000_000_000,
    100_000_000_000_000_000_000_000,
    1_000_000_000_000_000_000_000_000,
    10_000_000_000_000_000_000_000_000,
    100_000_000_000_000_000_000_000_000,
    1_000_000_000_000_000_000_000_000_000,
];

/// A fixed-point decimal number 128 bits wide
#[derive(Pod, Zeroable, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Number128(i128);

impl Number128 {
    pub const ONE: Self = Self(ONE);
    pub const ZERO: Self = Self(0i128);
    pub const MAX: Self = Self(i128::MAX);
    pub const MIN: Self = Self(i128::MIN);
    pub const BITS: u32 = i128::BITS;

    /// Convert this number to fit in a u64
    ///
    /// The precision of the number in the u64 is based on the
    /// exponent provided.
    pub fn as_u64(&self, exponent: impl Into<i32>) -> u64 {
        let extra_precision = PRECISION + exponent.into();
        let prec_value = POWERS_OF_TEN[extra_precision.unsigned_abs() as usize];

        let target_value = if extra_precision < 0 {
            self.0 * prec_value
        } else {
            self.0 / prec_value
        };

        if target_value > std::u64::MAX as i128 {
            panic!("cannot convert to u64 due to overflow");
        }

        if target_value < 0 {
            panic!("cannot convert to u64 because value < 0");
        }

        target_value as u64
    }

    /// Convert this number to a f64
    pub fn as_f64(&self) -> f64 {
        // i128::{MAX|MIN} fits within f64
        self.to_i128() as f64 / 10_000_000_000.0
    }

    /// Convert another integer
    pub fn from_decimal(value: impl Into<i128>, exponent: impl Into<i32>) -> Self {
        let extra_precision = PRECISION + exponent.into();
        let prec_value = POWERS_OF_TEN[extra_precision.unsigned_abs() as usize];

        if extra_precision < 0 {
            Self(value.into() / prec_value)
        } else {
            Self(value.into() * prec_value)
        }
    }

    /// Convert from basis points
    pub fn from_bps(basis_points: u16) -> Self {
        Self::from_decimal(basis_points, crate::BPS_EXPONENT)
    }

    /// Get the underlying 128-bit representation in bytes.
    /// Uses the target endianness of the caller
    pub fn into_bits(self) -> [u8; 16] {
        self.0.to_ne_bytes()
    }

    /// Read a number from a raw 128-bit representation, which was previously
    /// returned by a call to `into_bits`.
    /// Uses the target endianness of the caller
    pub fn from_bits(bits: [u8; 16]) -> Self {
        Self(i128::from_ne_bytes(bits))
    }

    /// Get the underlying i128 value
    pub fn to_i128(self) -> i128 {
        self.0
    }

    /// Create `Number128` from an `i128`
    pub fn from_i128(value: i128) -> Self {
        Self(value)
    }
}

impl std::fmt::Debug for Number128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(self, f)
    }
}

impl std::fmt::Display for Number128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // todo optimize
        let rem = self.0 % ONE;
        let decimal_digits = PRECISION as usize;
        // convert to abs to remove sign
        let rem_str = rem.abs().to_string();
        // regular padding like {:010} doesn't work with i128
        let decimals = "0".repeat(decimal_digits - rem_str.len()) + &*rem_str;
        let stripped_decimals = decimals.trim_end_matches('0');
        let pretty_decimals = if stripped_decimals.is_empty() {
            "0"
        } else {
            stripped_decimals
        };
        if self.0 < -ONE {
            let int = self.0 / ONE;
            write!(f, "{}.{}", int, pretty_decimals)?;
        } else if self.0 < 0 {
            write!(f, "-0.{}", pretty_decimals)?;
        } else if self.0 < ONE {
            write!(f, "0.{}", pretty_decimals)?;
        } else {
            let int = self.0 / ONE;
            write!(f, "{}.{}", int, pretty_decimals)?;
        }
        Ok(())
    }
}

impl Add<Number128> for Number128 {
    type Output = Self;

    fn add(self, rhs: Number128) -> Self::Output {
        Self(self.0.checked_add(rhs.0).unwrap())
    }
}

impl AddAssign<Number128> for Number128 {
    fn add_assign(&mut self, rhs: Number128) {
        self.0 = self.0.checked_add(rhs.0).unwrap();
    }
}

impl Sub<Number128> for Number128 {
    type Output = Self;

    fn sub(self, rhs: Number128) -> Self::Output {
        Self(self.0.checked_sub(rhs.0).unwrap())
    }
}

impl SubAssign<Number128> for Number128 {
    fn sub_assign(&mut self, rhs: Number128) {
        self.0 = self.0.checked_sub(rhs.0).unwrap();
    }
}

impl Mul<Number128> for Number128 {
    type Output = Number128;

    fn mul(self, rhs: Number128) -> Self::Output {
        Self(self.0.checked_mul(rhs.0).unwrap().div(ONE))
    }
}

impl MulAssign<Number128> for Number128 {
    fn mul_assign(&mut self, rhs: Number128) {
        self.0 = self.0 * rhs.0 / ONE;
    }
}

impl Div<Number128> for Number128 {
    type Output = Number128;

    fn div(self, rhs: Number128) -> Self::Output {
        Self(self.0.mul(ONE).div(rhs.0))
    }
}

impl DivAssign<Number128> for Number128 {
    fn div_assign(&mut self, rhs: Number128) {
        self.0 = self.0 * ONE / rhs.0;
    }
}

impl<T: Into<i128>> Mul<T> for Number128 {
    type Output = Number128;

    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0.mul(rhs.into()))
    }
}

impl<T: Into<i128>> Div<T> for Number128 {
    type Output = Number128;

    fn div(self, rhs: T) -> Self::Output {
        Self(self.0.div(rhs.into()))
    }
}

impl<T: Into<i128>> From<T> for Number128 {
    fn from(n: T) -> Self {
        Self::from_i128(n.into())
    }
}

impl num_traits::CheckedAdd for Number128 {
    fn checked_add(&self, v: &Self) -> Option<Self> {
        self.0.checked_add(v.0).map(|n| n.into())
    }
}

impl num_traits::CheckedDiv for Number128 {
    fn checked_div(&self, v: &Self) -> Option<Self> {
        self.0.checked_div(v.0).map(|n| n.into())
    }
}

impl num_traits::CheckedMul for Number128 {
    fn checked_mul(&self, v: &Self) -> Option<Self> {
        self.0.checked_mul(v.0).map(|n| n.into())
    }
}

impl num_traits::CheckedSub for Number128 {
    fn checked_sub(&self, v: &Self) -> Option<Self> {
        self.0.checked_sub(v.0).map(|n| n.into())
    }
}

impl Neg for Number128 {
    type Output = Number128;

    fn neg(self) -> Self::Output {
        Number128(-self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_equals_zero() {
        assert_eq!(Number128::ZERO, Number128::from_decimal(0, 0));
    }

    #[test]
    fn one_equals_one() {
        assert_eq!(Number128::ONE, Number128::from_decimal(1, 0));
    }

    #[test]
    fn negative_one_equals_negative_one() {
        assert_eq!(-Number128::ONE, Number128::from_decimal(-1, 0));
    }

    #[test]
    fn one_plus_one_equals_two() {
        assert_eq!(
            Number128::from_decimal(2, 0),
            Number128::ONE + Number128::ONE
        );
    }

    #[test]
    fn one_minus_one_equals_zero() {
        assert_eq!(Number128::ONE - Number128::ONE, Number128::ZERO);
    }

    #[test]
    fn one_times_one_equals_one() {
        assert_eq!(Number128::ONE, Number128::ONE * Number128::ONE);
    }

    #[test]
    fn one_divided_by_one_equals_one() {
        assert_eq!(Number128::ONE, Number128::ONE / Number128::ONE);
    }

    #[test]
    fn ten_div_100_equals_point_1() {
        assert_eq!(
            Number128::from_decimal(1, -1),
            Number128::from_decimal(1, 1) / Number128::from_decimal(100, 0)
        );
    }

    #[test]
    fn comparison() {
        let a = Number128::from_decimal(1000, -4);
        let b = Number128::from_decimal(10, -2);
        assert!(a >= b);

        let c = Number128::from_decimal(1001, -4);
        assert!(c > a);
        assert!(c > b);

        let d = Number128::from_decimal(9999999, -8);
        assert!(d < a);
        assert!(d < b);
        assert!(d < c);
        assert!(d <= d);

        assert_eq!(a.cmp(&b), std::cmp::Ordering::Equal);
        assert_eq!(a.cmp(&c), std::cmp::Ordering::Less);
        assert_eq!(a.cmp(&d), std::cmp::Ordering::Greater);
    }

    #[test]
    fn multiply_by_u64() {
        assert_eq!(
            Number128::from_decimal(3, 1),
            Number128::from_decimal(1, 1) * 3u64
        )
    }

    #[test]
    fn test_add_assign_101_2() {
        let mut a = Number128::from_decimal(101, 0);
        a += Number128::from_decimal(2, 0);
        assert_eq!(Number128::from_decimal(103, 0), a);
    }

    #[test]
    fn test_sub_assign_101_2() {
        let mut a = Number128::from_decimal(101, 0);
        a -= Number128::from_decimal(2, 0);
        assert_eq!(Number128::from_decimal(99, 0), a);
    }

    #[test]
    fn test_mul_assign_101_2() {
        let mut a = Number128::from_decimal(101, 0);
        a *= Number128::from_decimal(2, 0);
        assert_eq!(Number128::from_decimal(202, 0).0, a.0);
    }

    #[test]
    fn test_div_assign_101_2() {
        let mut a = Number128::from_decimal(101, 0);
        a /= Number128::from_decimal(2, 0);
        assert_eq!(Number128::from_decimal(505, -1), a);
    }

    #[test]
    fn test_div_assign_102_3() {
        let mut a = Number128::from_decimal(1, 1);
        a /= Number128::from_decimal(100, 0);
        assert_eq!(Number128::from_decimal(1, -1).0, a.0);
    }

    #[test]
    fn div_into_i128() {
        let a = Number128::from_decimal(1000, 0);
        let b = a / 500;
        assert_eq!(Number128::from_decimal(2, 0), b);

        let c = Number128::from_decimal(1000, -3);
        let d = c / 3;
        assert_eq!(Number128::from_decimal(3333333333i64, -10).0, d.0);
    }

    #[test]
    fn equality() {
        let a = Number128::from_decimal(1000, -4);
        let b = Number128::from_decimal(10, -2);
        assert_eq!(a, b);

        let c = Number128::from_decimal(-1000, -4);
        assert_ne!(a, c);
        assert_ne!(b, c);
    }

    #[test]
    fn as_u64() {
        let u64in = 31455;
        let a = Number128::from_decimal(u64in, -3);
        let b = a.as_u64(-3);
        assert_eq!(b, u64in);
    }

    #[test]
    #[should_panic = "cannot convert to u64 because value < 0"]
    fn as_u64_panic_neg() {
        let a = Number128::from_decimal(-10000, -3);
        a.as_u64(-3);
    }

    #[test]
    #[should_panic = "cannot convert to u64 due to overflow"]
    fn as_u64_panic_big() {
        let a = Number128::from_decimal(u64::MAX as i128 + 1, -3);
        a.as_u64(-3);
    }

    #[test]
    fn as_f64() {
        let n = Number128::from_bps(15000);
        assert_eq!(1.5, n.as_f64());

        // Test that conversion is within bounds and doesn't lose precision for min
        let n = Number128::MIN; // -170141183460469231731687303715884105728
        assert_eq!(-17014118346046923173168730371.5884105728, n.as_f64());

        // Test that conversion is within bounds and doesn't lose precision for max
        let n = Number128::MAX; // 170141183460469231731687303715884105727
        assert_eq!(17014118346046923173168730371.5884105727, n.as_f64());

        // More cases
        let n = Number128::from_bps(0) - Number128::from_bps(15000);
        assert_eq!(-1.5, n.as_f64());

        let n = Number128::from_decimal(12345678901i128, -10);
        assert_eq!(1.2345678901, n.as_f64());

        let n = Number128::from_decimal(-12345678901i128, -10);
        assert_eq!(-1.2345678901, n.as_f64());

        let n = Number128::from_decimal(-12345678901i128, -9);
        assert_eq!(-12.345678901, n.as_f64());

        let n = Number128::from_decimal(12345678901i128, -9);
        assert_eq!(12.345678901, n.as_f64());

        let n = Number128::from_decimal(ONE - 1, 1);
        assert_eq!(99999999990.0, n.as_f64());

        let n = Number128::from_decimal(12345678901i128, -13);
        assert_eq!(0.0012345678, n.as_f64());

        let n = Number128::from_decimal(-12345678901i128, -13);
        assert_eq!(-0.0012345678, n.as_f64());
    }

    #[test]
    fn display() {
        let a = Number128::from_bps(15000);
        assert_eq!("1.5", a.to_string().as_str());

        let a = Number128::from_bps(0) - Number128::from_bps(15000);
        assert_eq!("-1.5", a.to_string().as_str());

        let b = Number128::from_decimal(12345678901i128, -10);
        assert_eq!("1.2345678901", b.to_string().as_str());

        let b = Number128::from_decimal(-12345678901i128, -10);
        assert_eq!("-1.2345678901", b.to_string().as_str());

        let c = Number128::from_decimal(-12345678901i128, -9);
        assert_eq!("-12.345678901", c.to_string().as_str());

        let c = Number128::from_decimal(12345678901i128, -9);
        assert_eq!("12.345678901", c.to_string().as_str());

        let d = Number128::from_decimal(ONE - 1, 1);
        assert_eq!("99999999990.0", d.to_string().as_str());

        let e = Number128::from_decimal(12345678901i128, -13);
        assert_eq!("0.0012345678", e.to_string().as_str());

        let e = Number128::from_decimal(-12345678901i128, -13);
        assert_eq!("-0.0012345678", e.to_string().as_str());
    }

    #[test]
    fn into_bits() {
        let bits = Number128::from_decimal(1242, -3).into_bits();
        let number = Number128::from_bits(bits);

        assert_eq!(Number128::from_decimal(1242, -3), number);
    }
}
