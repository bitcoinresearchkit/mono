use std::ops::{Add, AddAssign};

use brk_types::CheckedSub;
use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{PartsPerMillion64, unlikely};

#[cfg(feature = "storage")]
use vecdb::Pco;

/// A price divided by a reference price, in parts per million (1,000,000 represents 1).
///
/// Finite values saturate at 4,294.967294; u32::MAX represents undefined.
/// Saturation is deliberately specific to price ratios, across all cohorts.
/// Non-finite inputs become undefined; finite inputs must be nonnegative
/// (a debug-checked precondition). PPM conversion rounds to nearest.
/// Serde preserves raw encoded bits; vector JSON emits null for undefined.
#[derive(
    Debug,
    Deref,
    Clone,
    Default,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
#[repr(transparent)]
pub struct PriceRatio(u32);

impl PriceRatio {
    const SCALE: u32 = 1_000_000;
    pub const MAX: Self = Self(u32::MAX - 1);
    pub const NAN: Self = Self(u32::MAX);

    pub const fn is_nan(self) -> bool {
        self.0 == u32::MAX
    }
}

impl From<usize> for PriceRatio {
    #[inline]
    fn from(raw: usize) -> Self {
        debug_assert!(raw < u32::MAX as usize, "PriceRatio raw value out of range");
        Self(raw as u32)
    }
}

impl Add for PriceRatio {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        if unlikely(self.is_nan() || rhs.is_nan()) {
            Self::NAN
        } else {
            Self(self.0.saturating_add(rhs.0).min(Self::MAX.0))
        }
    }
}

impl AddAssign for PriceRatio {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl CheckedSub for PriceRatio {
    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        if unlikely(self.is_nan() || rhs.is_nan()) {
            Some(Self::NAN)
        } else {
            self.0.checked_sub(rhs.0).map(Self)
        }
    }
}

impl From<f64> for PriceRatio {
    #[inline]
    fn from(value: f64) -> Self {
        if unlikely(!value.is_finite()) {
            return Self::NAN;
        }
        debug_assert!(value >= 0.0, "Negative PriceRatio: {value}");
        // Check before scaling so even f64::MAX saturates, not becomes undefined.
        if unlikely(value >= f64::from(Self::MAX)) {
            return Self::MAX;
        }
        Self((value * Self::SCALE as f64).round() as u32)
    }
}

impl From<PartsPerMillion64> for PriceRatio {
    #[inline]
    fn from(value: PartsPerMillion64) -> Self {
        if unlikely(value.is_nan()) {
            Self::NAN
        } else {
            Self(value.inner().min(Self::MAX.0 as u64) as u32)
        }
    }
}

impl From<PriceRatio> for f64 {
    #[inline]
    fn from(value: PriceRatio) -> Self {
        if unlikely(value.is_nan()) {
            f64::NAN
        } else {
            value.0 as f64 / PriceRatio::SCALE as f64
        }
    }
}

impl From<PriceRatio> for f32 {
    #[inline]
    fn from(value: PriceRatio) -> Self {
        if unlikely(value.is_nan()) {
            f32::NAN
        } else {
            value.0 as f32 / PriceRatio::SCALE as f32
        }
    }
}

brk_types::impl_int_format!(PriceRatio, nullable);
