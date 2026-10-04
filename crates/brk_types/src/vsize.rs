use std::{
    fmt::{Display, Formatter, Result},
    iter::Sum,
    ops::{Add, AddAssign, Mul, Sub, SubAssign},
};

use derive_more::Deref;
use itoa::Buffer;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CheckedSub, Weight};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Virtual size in vbytes (weight / 4, rounded up). Max block vsize is ~1,000,000 vB.
#[derive(
    Debug, Default, Deref, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
#[cfg_attr(feature = "schemars", schemars(
    example = &110,
    example = &140,
    example = &225,
    example = &500_000,
    example = &998_368
))]
pub struct VSize(u64);

impl VSize {
    /// Maximum block vsize (1M vB), the policy budget the partitioner targets.
    pub const MAX_BLOCK: Self = Self(1_000_000);

    #[inline]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[inline]
    pub fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl From<u64> for VSize {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<u32> for VSize {
    #[inline]
    fn from(value: u32) -> Self {
        Self(u64::from(value))
    }
}

impl From<VSize> for u64 {
    #[inline]
    fn from(value: VSize) -> Self {
        value.0
    }
}

impl From<Weight> for VSize {
    #[inline]
    fn from(weight: Weight) -> Self {
        Self(weight.to_vbytes_ceil())
    }
}

impl From<usize> for VSize {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}

impl From<f64> for VSize {
    #[inline]
    fn from(value: f64) -> Self {
        let value = value.max(0.0);
        debug_assert!(value.fract() == 0.0, "VSize must be an integer");
        Self(value as u64)
    }
}

impl From<VSize> for f64 {
    #[inline]
    fn from(value: VSize) -> Self {
        value.0 as f64
    }
}

impl Add for VSize {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for VSize {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sub for VSize {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for VSize {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Mul<u32> for VSize {
    type Output = Self;
    fn mul(self, rhs: u32) -> Self::Output {
        Self(self.0 * u64::from(rhs))
    }
}

impl Sum for VSize {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|v| v.0).sum())
    }
}

impl CheckedSub for VSize {
    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for VSize {
    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Display for VSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for VSize {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
