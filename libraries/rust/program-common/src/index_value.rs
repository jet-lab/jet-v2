use std::ops::{SubAssign, AddAssign};

use anchor_lang::{AnchorDeserialize, AnchorSerialize};
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Default, AnchorDeserialize, AnchorSerialize, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct IndexValue(pub u64);

impl IndexValue {
    pub fn as_u64(self) -> u64 {
        self.into()
    }

    pub fn as_usize(self) -> usize {
        self.into()
    }
}

impl From<IndexValue> for usize {
    #[cfg(target_arch = "wasm32")]
    fn from(value: IndexValue) -> Self {
        value.0.try_into().expect("wasm32 target 32-bit usize unsuitable")
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn from(value: IndexValue) -> Self {
        value.0 as Self
    }
}

impl From<usize> for IndexValue {
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}

impl From<IndexValue> for u64 {
    fn from(value: IndexValue) -> Self {
        value.0
    }
}

impl SubAssign<u64> for IndexValue {
    fn sub_assign(&mut self, rhs: u64) {
        self.0 -= rhs
    }
}

impl AddAssign<u64> for IndexValue {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs
    }
}
