use std::{
    fmt::{Display, Formatter, Result},
    iter::Sum,
    ops::{Add, AddAssign, Div, Sub, SubAssign},
};

use bitcoin::Weight as BitcoinWeight;
use derive_more::Deref;
use itoa::Buffer;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CheckedSub, VSize};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Weight in weight units (WU). Max block weight is 4,000,000 WU.
#[derive(
    Debug, Default, Deref, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
#[cfg_attr(feature = "schemars", schemars(
    example = &396,
    example = &561,
    example = &900,
    example = &2_000_000,
    example = &3_993_472
))]
pub struct Weight(u32);

impl Weight {
    /// Maximum block weight in Bitcoin (4 million weight units).
    /// Note: Pre-SegWit 1MB blocks have weight = size * 4 = 4M, so this is consistent across all blocks.
    pub const MAX_BLOCK: Self = Self(BitcoinWeight::MAX_BLOCK.to_wu() as u32);

    /// Compute weight from base size and total size.
    #[inline]
    pub fn from_sizes(base_size: u32, total_size: u32) -> Self {
        let base_size = u64::from(base_size);
        let witness_size = u64::from(total_size) - base_size;
        let value = (BitcoinWeight::from_non_witness_data_size(base_size)
            + BitcoinWeight::from_witness_data_size(witness_size))
        .to_wu();
        debug_assert!(u32::try_from(value).is_ok());
        Self(value as u32)
    }

    pub fn to_vbytes_ceil(&self) -> u64 {
        BitcoinWeight::from(*self).to_vbytes_ceil()
    }

    /// Returns block fullness as a ratio (0–1+) relative to MAX_BLOCK.
    #[inline]
    pub fn fullness(&self) -> f32 {
        (self.0 as f64 / Self::MAX_BLOCK.0 as f64) as f32
    }
}

impl From<BitcoinWeight> for Weight {
    #[inline]
    fn from(value: BitcoinWeight) -> Self {
        let value = value.to_wu();
        debug_assert!(u32::try_from(value).is_ok());
        Self(value as u32)
    }
}

impl From<Weight> for BitcoinWeight {
    #[inline]
    fn from(value: Weight) -> Self {
        Self::from_wu(u64::from(value.0))
    }
}

impl From<VSize> for Weight {
    /// Convert virtual bytes to weight units: `weight = vbytes * WITNESS_SCALE_FACTOR`.
    #[inline]
    fn from(vsize: VSize) -> Self {
        Self::from(BitcoinWeight::from_vb_unchecked(*vsize))
    }
}

impl From<u32> for Weight {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<u64> for Weight {
    #[inline]
    fn from(value: u64) -> Self {
        debug_assert!(u32::try_from(value).is_ok());
        Self(value as u32)
    }
}

impl From<usize> for Weight {
    #[inline]
    fn from(value: usize) -> Self {
        debug_assert!(u32::try_from(value).is_ok());
        Self(value as u32)
    }
}

impl From<f64> for Weight {
    #[inline]
    fn from(value: f64) -> Self {
        let value = value.max(0.0);
        debug_assert!(value <= f64::from(u32::MAX));
        Self(value as u32)
    }
}

impl From<Weight> for f64 {
    #[inline]
    fn from(value: Weight) -> Self {
        value.0 as f64
    }
}

impl From<Weight> for u32 {
    #[inline]
    fn from(value: Weight) -> Self {
        value.0
    }
}

impl From<Weight> for u64 {
    #[inline]
    fn from(value: Weight) -> Self {
        u64::from(value.0)
    }
}

impl Add for Weight {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let value = u64::from(self.0) + u64::from(rhs.0);
        debug_assert!(u32::try_from(value).is_ok());
        Self(value as u32)
    }
}

impl AddAssign for Weight {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sum for Weight {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let value: u64 = iter.map(u64::from).sum();
        debug_assert!(u32::try_from(value).is_ok());
        Self(value as u32)
    }
}

impl Sub for Weight {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Weight {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Div<usize> for Weight {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self((u64::from(self.0) / rhs as u64) as u32)
    }
}

impl Div<Weight> for Weight {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl CheckedSub for Weight {
    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for Weight {
    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Display for Weight {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Weight {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
