use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div},
};

use bitcoin::Weight as BitcoinWeight;
use derive_more::{Deref, From, Into, Sub, SubAssign};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CheckedSub, VSize};

#[cfg(feature = "storage")]
use vecdb::Pco;

/// A transaction's or block's weight in 32 bits: the storage form of `Weight` (a block holds at most
/// 4,000,000 WU).
#[derive(
    Debug,
    Default,
    Deref,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Sub,
    SubAssign,
    From,
    Into,
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
pub struct Weight32(u32);

impl From<BitcoinWeight> for Weight32 {
    #[inline]
    fn from(value: BitcoinWeight) -> Self {
        let value = value.to_wu();
        debug_assert!(u32::try_from(value).is_ok());
        Self(value as u32)
    }
}

impl From<Weight32> for BitcoinWeight {
    #[inline]
    fn from(value: Weight32) -> Self {
        Self::from_wu(u64::from(value.0))
    }
}

impl From<VSize> for Weight32 {
    /// Convert virtual bytes to weight units: `weight = vbytes * WITNESS_SCALE_FACTOR`.
    #[inline]
    fn from(vsize: VSize) -> Self {
        Self::from(BitcoinWeight::from_vb_unchecked(*vsize))
    }
}

impl From<u64> for Weight32 {
    #[inline]
    fn from(value: u64) -> Self {
        debug_assert!(u32::try_from(value).is_ok());
        Self(value as u32)
    }
}

impl From<usize> for Weight32 {
    #[inline]
    fn from(value: usize) -> Self {
        debug_assert!(u32::try_from(value).is_ok());
        Self(value as u32)
    }
}

impl From<f64> for Weight32 {
    #[inline]
    fn from(value: f64) -> Self {
        let value = value.max(0.0);
        debug_assert!(value <= f64::from(u32::MAX));
        Self(value as u32)
    }
}

impl From<Weight32> for f64 {
    #[inline]
    fn from(value: Weight32) -> Self {
        value.0 as f64
    }
}

impl From<Weight32> for u64 {
    #[inline]
    fn from(value: Weight32) -> Self {
        u64::from(value.0)
    }
}

impl Add for Weight32 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let value = u64::from(self.0) + u64::from(rhs.0);
        debug_assert!(u32::try_from(value).is_ok());
        Self(value as u32)
    }
}

impl AddAssign for Weight32 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sum for Weight32 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let value: u64 = iter.map(u64::from).sum();
        debug_assert!(u32::try_from(value).is_ok());
        Self(value as u32)
    }
}

impl Div<Weight32> for Weight32 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl CheckedSub for Weight32 {
    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

crate::impl_int_format!(Weight32);
