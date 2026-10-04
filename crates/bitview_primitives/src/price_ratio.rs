use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    ops::{Add, AddAssign, Div},
};

use brk_types::CheckedSub;
use derive_more::Deref;
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{PartsPerMillion64, StoredF32, unlikely};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Spot price divided by a reference price, encoded in parts per million.
/// Finite values saturate at 4,294.967294; u32::MAX represents undefined.
/// Saturation is deliberately specific to price ratios, across all cohorts.
/// Non-finite inputs become undefined; finite inputs must be nonnegative
/// (a debug-checked precondition). PPM conversion rounds to nearest, matching
/// the existing price ratios.
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

impl Div<usize> for PriceRatio {
    type Output = Self;

    #[inline]
    fn div(self, rhs: usize) -> Self {
        if unlikely(self.is_nan() || rhs == 0) {
            Self::NAN
        } else {
            Self((self.0 as usize / rhs) as u32)
        }
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
#[cfg(feature = "storage")]
impl VecdbCheckedSub for PriceRatio {
    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
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

impl From<PriceRatio> for StoredF32 {
    #[inline]
    fn from(value: PriceRatio) -> Self {
        Self::from(f32::from(value))
    }
}

impl Display for PriceRatio {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut buf = Buffer::new();
        f.write_str(buf.format(self.0))
    }
}

#[cfg(feature = "storage")]
impl Formattable for PriceRatio {
    const UNDEFINED: bool = true;

    #[inline]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut value = Buffer::new();
        buf.extend_from_slice(value.format(self.0).as_bytes());
    }

    #[inline]
    fn fmt_json(&self, buf: &mut Vec<u8>) {
        if unlikely(self.is_nan()) {
            buf.extend_from_slice(b"null");
        } else {
            self.write_to(buf);
        }
    }

    /// Undefined: an empty CSV cell (JSON writes `null`).
    #[inline(always)]
    fn fmt_csv(&self, f: &mut String) -> std::fmt::Result {
        if !unlikely(self.is_nan()) {
            self.fmt_into(f);
        }
        Ok(())
    }
}
