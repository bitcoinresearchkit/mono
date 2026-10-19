use derive_more::{Add, AddAssign, From, Into, Sub, SubAssign};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Dollars;
use crate::CheckedSub;

#[cfg(feature = "storage")]
use vecdb::Pco;

/// A signed amount in US cents (100 cents = 1 USD).
#[derive(
    Debug,
    Default,
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
    From,
    Into,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct CentsSigned(i64);

impl CentsSigned {
    #[inline]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    #[inline]
    pub(crate) fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

impl From<Dollars> for CentsSigned {
    #[inline]
    fn from(value: Dollars) -> Self {
        Self::from(*value * 100.0)
    }
}

impl From<CentsSigned> for Dollars {
    #[inline]
    fn from(value: CentsSigned) -> Self {
        Self::from(value.0 as f64 / 100.0)
    }
}

impl From<CentsSigned> for f64 {
    #[inline]
    fn from(value: CentsSigned) -> Self {
        value.0 as f64
    }
}

/// Rounds to the nearest cent.
impl From<f64> for CentsSigned {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value.round() as i64)
    }
}

impl From<u64> for CentsSigned {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value as i64)
    }
}

impl From<CentsSigned> for usize {
    #[inline]
    fn from(value: CentsSigned) -> Self {
        debug_assert!(value.0 >= 0, "negative CentsSigned to usize");
        value.0 as usize
    }
}

impl From<usize> for CentsSigned {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as i64)
    }
}

impl From<CentsSigned> for u64 {
    #[inline]
    fn from(value: CentsSigned) -> Self {
        debug_assert!(value.0 >= 0, "negative CentsSigned to u64");
        value.0 as u64
    }
}

impl From<CentsSigned> for i128 {
    #[inline]
    fn from(value: CentsSigned) -> Self {
        value.0 as i128
    }
}

impl From<i128> for CentsSigned {
    #[inline]
    fn from(value: i128) -> Self {
        debug_assert!(
            value >= i64::MIN as i128 && value <= i64::MAX as i128,
            "i128 overflow to CentsSigned"
        );
        Self(value as i64)
    }
}

impl From<u128> for CentsSigned {
    #[inline]
    fn from(value: u128) -> Self {
        debug_assert!(value <= i64::MAX as u128, "u128 overflow to CentsSigned");
        Self(value as i64)
    }
}

impl From<CentsSigned> for u128 {
    #[inline]
    fn from(value: CentsSigned) -> Self {
        debug_assert!(value.0 >= 0, "negative CentsSigned to u128");
        value.0 as u128
    }
}

impl CheckedSub for CentsSigned {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.checked_sub(rhs)
    }
}

crate::impl_int_format!(CentsSigned);
