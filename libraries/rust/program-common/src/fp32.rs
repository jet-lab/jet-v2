use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use bytemuck::{Pod, Zeroable};

pub const FP32_ONE: u128 = 1 << 32;

#[derive(Pod, Zeroable, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Fp32(u128);

impl Fp32 {
    pub const ONE: Self = Self(FP32_ONE);
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(u128::MAX);
    pub const MIN: Self = Self(u128::MIN);

    /// wraps a u128 value without any logical conversions
    pub fn wrap_u128(n: u128) -> Self {
        Self(n)
    }

    /// returns fixed-point to decimal representation
    pub fn as_decimal_u64(self) -> Option<u64> {
        let res = self.0 / Self::ONE.0;
        if res > u64::MAX as u128 {
            None
        } else {
            Some(res as u64)
        }
    }

    /// returns fixed-point to decimal representation, rounded up
    pub fn as_decimal_u64_ceil(self) -> Option<u64> {
        let add_one = (!(self.0 as u32)).wrapping_add(1) as u128;
        self.0
            .checked_add(add_one)
            .and_then(|n| Self(n).as_decimal_u64())
    }

    /// Keeps representation as a fixed-point 32 number
    pub fn downcast_u64(self) -> Option<u64> {
        if self.0 > u64::MAX as u128 {
            None
        } else {
            Some(self.0 as u64)
        }
    }

    /// multiplies self with rhs yielding a decimal u64
    pub fn decimal_u64_mul(&self, rhs: u64) -> Option<u64> {
        (*self * rhs).as_decimal_u64()
    }

    /// divides self with rhs yielding a u64
    pub fn u64_div(&self, rhs: u64) -> Option<u64> {
        (*self / rhs).as_decimal_u64()
    }

    /// upcasts and wraps an existing 64-bit fp32
    pub fn upcast_fp32(fp: u64) -> Self {
        Self(fp as u128)
    }
}

impl<T: Into<u128>> From<T> for Fp32 {
    fn from(n: T) -> Fp32 {
        Fp32(n.into() * FP32_ONE)
    }
}

impl Add<Fp32> for Fp32 {
    type Output = Fp32;

    fn add(self, rhs: Fp32) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl Sub<Fp32> for Fp32 {
    type Output = Fp32;

    fn sub(self, rhs: Fp32) -> Self::Output {
        Self(self.0.sub(rhs.0))
    }
}

impl AddAssign<Fp32> for Fp32 {
    fn add_assign(&mut self, rhs: Fp32) {
        self.0.add_assign(rhs.0)
    }
}

impl SubAssign<Fp32> for Fp32 {
    fn sub_assign(&mut self, rhs: Fp32) {
        self.0.sub_assign(rhs.0)
    }
}

impl MulAssign<Fp32> for Fp32 {
    fn mul_assign(&mut self, rhs: Fp32) {
        self.0.mul_assign(rhs.0);
        self.0.div_assign(Self::ONE.0);
    }
}

impl Mul<Fp32> for Fp32 {
    type Output = Fp32;

    fn mul(self, rhs: Fp32) -> Self::Output {
        Self(self.0.mul(rhs.0).div(FP32_ONE))
    }
}

impl Div<Fp32> for Fp32 {
    type Output = Fp32;

    fn div(self, rhs: Fp32) -> Self::Output {
        Self(self.0.mul(FP32_ONE).div(rhs.0))
    }
}

impl<T: Into<u128>> Mul<T> for Fp32 {
    type Output = Fp32;

    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0.mul(rhs.into()))
    }
}

impl<T: Into<u128>> Div<T> for Fp32 {
    type Output = Fp32;

    fn div(self, rhs: T) -> Self::Output {
        Self(self.0.div(rhs.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_equals_zero() {
        assert_eq!(Fp32::ZERO, Fp32::from(0u64));
    }

    #[test]
    fn one_equals_one() {
        assert_eq!(Fp32::ONE, Fp32::from(1u64));
    }

    #[test]
    fn one_plus_one_equals_two() {
        assert_eq!(Fp32::from(2u64), Fp32::ONE + Fp32::ONE);
    }

    #[test]
    fn one_minus_one_equals_zero() {
        assert_eq!(Fp32::ONE - Fp32::from(1u64), Fp32::ZERO);
    }

    #[test]
    fn one_times_one_equals_one() {
        assert_eq!(Fp32::ONE, Fp32::ONE * Fp32::ONE);
        assert_eq!(Fp32::ONE, Fp32::from(1u64) * Fp32::ONE);
    }

    #[test]
    fn one_divided_by_one_equals_one() {
        assert_eq!(Fp32::ONE, Fp32::ONE / Fp32::ONE);
        assert_eq!(Fp32::ONE, Fp32::from(1u64) / Fp32::ONE);
    }

    #[test]
    fn one_thousand_div_100_equals_ten() {
        assert_eq!(Fp32::from(10u64), Fp32::from(1_000u64) / Fp32::from(100u64));
    }

    #[test]
    fn multiply_by_u64() {
        assert_eq!(Fp32::from(3u64), Fp32::from(1u64) * 3u64)
    }

    #[test]
    fn test_add_assign_101_2() {
        let mut a = Fp32::from(101u64);
        a += Fp32::from(2u64);
        assert_eq!(Fp32::from(103u64), a);
    }

    #[test]
    fn test_sub_assign_101_2() {
        let mut a = Fp32::from(101u64);
        a -= Fp32::from(2u64);
        assert_eq!(Fp32::from(99u64), a);
    }

    #[test]
    fn test_mul_assign_101_2() {
        let mut a = Fp32::from(101u64);
        a *= Fp32::from(2u64);
        assert_eq!(Fp32::from(202u64), a);
    }

    #[test]
    fn test_div_assign_101_2() {
        let mut a = Fp32::from(101u64);
        a = a / Fp32::from(2u64);
        assert_eq!(Fp32::from(505u64) / Fp32::from(10u64), a);
    }

    #[test]
    fn as_u64() {
        let u64in = 31455u64;
        let a = Fp32::from(u64in);
        let b = a.as_decimal_u64().unwrap();
        assert_eq!(b, u64in);
    }

    #[test]
    #[should_panic]
    fn as_u64_panic_big() {
        let a = Fp32::from(u64::MAX as u128 + 1);
        a.as_decimal_u64().unwrap();
    }

    #[test]
    fn u64_mul() {
        let a = 100u64;
        let fp = Fp32::from(10u64);

        assert_eq!(fp.decimal_u64_mul(a).unwrap(), 1_000);
    }

    #[test]
    fn u64_div() {
        let a = 10u64;
        let fp = Fp32::from(100u64);

        assert_eq!(fp.u64_div(a).unwrap(), 10u64)
    }

    #[test]
    fn up_and_downcasting() {
        let fp32_u64 = 10 * (FP32_ONE as u64);
        let up = Fp32::upcast_fp32(fp32_u64);

        assert_eq!(up.downcast_u64().unwrap(), fp32_u64);
    }
}
