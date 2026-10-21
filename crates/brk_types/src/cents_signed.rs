use std::ops::{Add, AddAssign, Sub, SubAssign};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Dollars;
use crate::{CheckedSub, unlikely};

#[cfg(feature = "storage")]
use vecdb::Pco;

/// A signed amount in US cents (100 cents = 1 USD).
///
/// `i64::MIN` is reserved as a NaN sentinel.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct CentsSigned(i64);

impl CentsSigned {
    pub const NAN: Self = Self(i64::MIN);

    #[inline]
    pub const fn new(value: i64) -> Self {
        debug_assert!(
            value != i64::MIN,
            "i64::MIN is reserved as CentsSigned::NAN"
        );
        Self(value)
    }

    #[inline]
    pub const fn is_nan(self) -> bool {
        self.0 == i64::MIN
    }

    /// NaN when either is; else the difference, none on overflow (or when it'd be the sentinel).
    #[inline]
    pub(crate) fn checked_sub(self, rhs: Self) -> Option<Self> {
        if unlikely(self.is_nan() || rhs.is_nan()) {
            Some(Self::NAN)
        } else {
            self.0
                .checked_sub(rhs.0)
                .filter(|&value| value != i64::MIN)
                .map(Self)
        }
    }
}

/// NaN when either is (an overflow panics, as `i64`'s).
impl Add for CentsSigned {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        if unlikely(self.is_nan() || rhs.is_nan()) {
            Self::NAN
        } else {
            Self::new(self.0 + rhs.0)
        }
    }
}

impl AddAssign for CentsSigned {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

/// NaN when either is (an overflow panics, as `i64`'s).
impl Sub for CentsSigned {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        if unlikely(self.is_nan() || rhs.is_nan()) {
            Self::NAN
        } else {
            Self::new(self.0 - rhs.0)
        }
    }
}

impl SubAssign for CentsSigned {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl From<i64> for CentsSigned {
    #[inline]
    fn from(value: i64) -> Self {
        Self::new(value)
    }
}

impl From<CentsSigned> for i64 {
    #[inline]
    fn from(value: CentsSigned) -> Self {
        debug_assert!(!value.is_nan(), "a NaN CentsSigned to i64");
        value.0
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
        if unlikely(value.is_nan()) {
            Dollars::NAN
        } else {
            Self::from(value.0 as f64 / 100.0)
        }
    }
}

impl From<CentsSigned> for f64 {
    #[inline]
    fn from(value: CentsSigned) -> Self {
        if unlikely(value.is_nan()) {
            f64::NAN
        } else {
            value.0 as f64
        }
    }
}

/// Rounds to the nearest cent; not finite, NaN.
impl From<f64> for CentsSigned {
    #[inline]
    fn from(value: f64) -> Self {
        let value = value.round();
        if unlikely(!value.is_finite()) {
            Self::NAN
        } else {
            // (The saturating cast's low end is the sentinel: the next above it.)
            Self((value as i64).max(i64::MIN + 1))
        }
    }
}

impl From<u64> for CentsSigned {
    #[inline]
    fn from(value: u64) -> Self {
        debug_assert!(value <= i64::MAX as u64, "u64 overflow to CentsSigned");
        Self::new(value as i64)
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
        debug_assert!(value <= i64::MAX as usize, "usize overflow to CentsSigned");
        Self::new(value as i64)
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
        debug_assert!(!value.is_nan(), "a NaN CentsSigned to i128");
        value.0 as i128
    }
}

impl From<i128> for CentsSigned {
    #[inline]
    fn from(value: i128) -> Self {
        debug_assert!(
            value > i64::MIN as i128 && value <= i64::MAX as i128,
            "i128 overflow to CentsSigned"
        );
        Self::new(value as i64)
    }
}

impl From<u128> for CentsSigned {
    #[inline]
    fn from(value: u128) -> Self {
        debug_assert!(value <= i64::MAX as u128, "u128 overflow to CentsSigned");
        Self::new(value as i64)
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

crate::impl_int_format!(CentsSigned, nullable);
