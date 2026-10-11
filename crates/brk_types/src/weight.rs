use std::{
    fmt::{Display, Formatter, Result},
    iter::Sum,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use derive_more::Deref;
use itoa::Buffer;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use bitcoin::Weight as BitcoinWeight;

use crate::{CheckedSub, VSize, Weight32};

#[cfg(feature = "storage")]
use vecdb::{CheckedSub as VecdbCheckedSub, Formattable, Pco};

/// Weight in weight units (WU): non-witness bytes count four and witness bytes one. A block holds
/// at most 4,000,000 WU; sums over many blocks use the full 64 bits.
#[derive(
    Debug, Default, Deref, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
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
pub struct Weight(u64);

impl Weight {
    pub const ZERO: Self = Self(0);

    /// Maximum block weight in Bitcoin (4 million weight units). Pre-SegWit 1 MB blocks weigh
    /// size * 4 = 4M too, so the limit is consistent across all blocks.
    pub const MAX_BLOCK: Self = Self(BitcoinWeight::MAX_BLOCK.to_wu());

    #[inline]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Weight from a transaction's base (non-witness) size and total size.
    #[inline]
    pub fn from_sizes(base_size: u32, total_size: u32) -> Self {
        let base_size = u64::from(base_size);
        let witness_size = u64::from(total_size) - base_size;
        Self(
            (BitcoinWeight::from_non_witness_data_size(base_size)
                + BitcoinWeight::from_witness_data_size(witness_size))
            .to_wu(),
        )
    }

    pub fn to_vbytes_ceil(&self) -> u64 {
        self.0.div_ceil(4)
    }

    /// Block fullness as a ratio (0–1+) of `MAX_BLOCK`.
    #[inline]
    pub fn fullness(&self) -> f32 {
        (self.0 as f64 / Self::MAX_BLOCK.0 as f64) as f32
    }
}

impl From<BitcoinWeight> for Weight {
    #[inline]
    fn from(value: BitcoinWeight) -> Self {
        Self(value.to_wu())
    }
}

impl From<Weight> for BitcoinWeight {
    #[inline]
    fn from(value: Weight) -> Self {
        Self::from_wu(value.0)
    }
}

impl From<VSize> for Weight {
    /// Virtual bytes to weight units: `weight = vbytes * WITNESS_SCALE_FACTOR`.
    #[inline]
    fn from(vsize: VSize) -> Self {
        Self::from(BitcoinWeight::from_vb_unchecked(*vsize))
    }
}

/// Narrowing to the storage form: a transaction's or block's weight fits 32 bits (checked in dev
/// builds).
impl From<Weight> for Weight32 {
    #[inline]
    fn from(value: Weight) -> Self {
        debug_assert!(
            u32::try_from(value.0).is_ok(),
            "{} doesn't fit Weight32",
            value.0
        );
        Self::from(value.0 as u32)
    }
}

impl From<u64> for Weight {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<Weight> for u64 {
    #[inline]
    fn from(value: Weight) -> Self {
        value.0
    }
}

impl From<Weight32> for Weight {
    #[inline]
    fn from(value: Weight32) -> Self {
        Self(u64::from(u32::from(value)))
    }
}

impl From<usize> for Weight {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}

/// Saturating (`as` from a float clamps and maps NaN to zero).
impl From<f64> for Weight {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value as u64)
    }
}

impl From<Weight> for f64 {
    #[inline]
    fn from(value: Weight) -> Self {
        value.0 as f64
    }
}

impl Add for Weight {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Weight {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Weight {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Weight {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Sum for Weight {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|value| value.0).sum())
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
        self.0.checked_sub(rhs.0).map(Self)
    }
}

impl Display for Weight {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(Buffer::new().format(self.0))
    }
}

#[cfg(feature = "storage")]
impl Formattable for Weight {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(Buffer::new().format(self.0).as_bytes());
    }
}
