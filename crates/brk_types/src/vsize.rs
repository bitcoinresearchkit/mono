use derive_more::{Add, AddAssign, Deref, From, Into, Sub, SubAssign, Sum};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CheckedSub, Weight, Weight32};

#[cfg(feature = "storage")]
use vecdb::Pco;

/// Virtual size in vbytes (weight / 4, rounded up). Max block vsize is ~1,000,000 vB.
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
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Sum,
    From,
    Into,
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

impl From<u32> for VSize {
    #[inline]
    fn from(value: u32) -> Self {
        Self(u64::from(value))
    }
}

impl From<Weight32> for VSize {
    #[inline]
    fn from(weight: Weight32) -> Self {
        Self::from(Weight::from(weight))
    }
}

impl From<Weight> for VSize {
    #[inline]
    fn from(weight: Weight) -> Self {
        Self(u64::from(weight).div_ceil(4))
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

impl CheckedSub for VSize {
    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

crate::impl_int_format!(VSize);
