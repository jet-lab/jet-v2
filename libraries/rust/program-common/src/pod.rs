use std::{
    mem::size_of,
    ops::{Deref, DerefMut},
};

use bytemuck::{Pod, Zeroable};
use static_assertions::const_assert_eq;

/// Allows any sized byte array to be Pod.
///
/// For some reason, bytemuck only implements certain sizes as Pod.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct PodBytes<const N: usize>(pub [u8; N]);
unsafe impl<const N: usize> Zeroable for PodBytes<N> {}
unsafe impl<const N: usize> Pod for PodBytes<N> {}
const_assert_eq!(3, size_of::<PodBytes<3>>());
const_assert_eq!(123, size_of::<PodBytes<123>>());
const_assert_eq!(7432, size_of::<PodBytes<7432>>());

impl<const N: usize> Default for PodBytes<N> {
    fn default() -> Self {
        Self([0; N])
    }
}

impl<const N: usize> From<[u8; N]> for PodBytes<N> {
    fn from(value: [u8; N]) -> Self {
        Self(value)
    }
}

impl<const N: usize> From<PodBytes<N>> for [u8; N] {
    fn from(val: PodBytes<N>) -> [u8; N] {
        val.0
    }
}

impl<const N: usize> Deref for PodBytes<N> {
    type Target = [u8; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for PodBytes<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Pod version of bool compatible with any bit pattern.
/// - Any bit pattern other than 0 evaluates to true.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, serde::Deserialize, serde::Serialize,
)]
#[repr(transparent)]
pub struct PodBool(u8);
const_assert_eq!(1, size_of::<PodBool>());

impl PodBool {
    pub fn as_bool(&self) -> bool {
        self.0 != 0
    }
}

impl From<bool> for PodBool {
    fn from(value: bool) -> Self {
        if value {
            Self(1)
        } else {
            Self(0)
        }
    }
}
impl From<PodBool> for bool {
    fn from(val: PodBool) -> Self {
        val.0 != 0
    }
}
