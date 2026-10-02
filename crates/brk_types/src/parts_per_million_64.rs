use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Div, Sub},
};

use derive_more::Deref;
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::StoredF32;
use crate::{CheckedSub, unlikely};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Unsigned parts per million stored as u64.
/// One unit is 0.000001. Range: 0–18,446,744,073,709.551614.
/// Use for precise wide-range ratios.
/// `u64::MAX` is reserved as a NaN sentinel.
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
pub struct PartsPerMillion64(u64);

impl PartsPerMillion64 {
    const ZERO: Self = Self(0);
    pub const NAN: Self = Self(u64::MAX);

    #[inline]
    pub const fn new(value: u64) -> Self {
        debug_assert!(value != u64::MAX, "u64::MAX is reserved as NaN sentinel");
        Self(value)
    }

    #[inline(always)]
    pub(crate) const fn inner(self) -> u64 {
        self.0
    }

    #[inline]
    pub(crate) fn is_nan(self) -> bool {
        self.0 == u64::MAX
    }

    #[inline]
    pub fn to_f32(self) -> f32 {
        if unlikely(self.is_nan()) {
            f32::NAN
        } else {
            self.0 as f32 / 1_000_000.0
        }
    }
}

impl From<usize> for PartsPerMillion64 {
    #[inline]
    fn from(value: usize) -> Self {
        debug_assert!(
            value < u64::MAX as usize,
            "usize out of PartsPerMillion64 range: {value}"
        );
        Self(value as u64)
    }
}

impl From<u64> for PartsPerMillion64 {
    #[inline]
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl From<PartsPerMillion64> for u64 {
    #[inline]
    fn from(value: PartsPerMillion64) -> Self {
        value.0
    }
}

impl From<f32> for PartsPerMillion64 {
    #[inline]
    fn from(value: f32) -> Self {
        if unlikely(!value.is_finite()) {
            return Self::NAN;
        }
        Self::from(value as f64)
    }
}

impl From<StoredF32> for PartsPerMillion64 {
    #[inline]
    fn from(value: StoredF32) -> Self {
        Self::from(*value)
    }
}

impl From<f64> for PartsPerMillion64 {
    #[inline]
    fn from(value: f64) -> Self {
        if unlikely(!value.is_finite()) {
            return Self::NAN;
        }
        let scaled = (value * 1_000_000.0).round();
        if scaled <= 0.0 {
            Self::ZERO
        } else if scaled >= u64::MAX as f64 {
            Self(u64::MAX - 1)
        } else {
            Self(scaled as u64)
        }
    }
}

impl From<PartsPerMillion64> for f64 {
    #[inline]
    fn from(value: PartsPerMillion64) -> Self {
        if unlikely(value.0 == u64::MAX) {
            f64::NAN
        } else {
            value.0 as f64 / 1_000_000.0
        }
    }
}

impl From<PartsPerMillion64> for f32 {
    #[inline]
    fn from(value: PartsPerMillion64) -> Self {
        value.to_f32()
    }
}

impl From<PartsPerMillion64> for StoredF32 {
    #[inline]
    fn from(value: PartsPerMillion64) -> Self {
        StoredF32::from(value.to_f32())
    }
}

impl Add for PartsPerMillion64 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        if unlikely(self.0 == u64::MAX || rhs.0 == u64::MAX) {
            Self::NAN
        } else {
            Self(self.0 + rhs.0)
        }
    }
}

impl Sub for PartsPerMillion64 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        if unlikely(self.0 == u64::MAX || rhs.0 == u64::MAX) {
            Self::NAN
        } else {
            Self(self.0 - rhs.0)
        }
    }
}

impl AddAssign for PartsPerMillion64 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Div<usize> for PartsPerMillion64 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: usize) -> Self::Output {
        if unlikely(self.0 == u64::MAX) {
            Self::NAN
        } else {
            debug_assert!(rhs <= u64::MAX as usize, "divisor out of u64 range: {rhs}");
            Self(self.0 / rhs as u64)
        }
    }
}

impl CheckedSub for PartsPerMillion64 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        if unlikely(self.0 == u64::MAX || rhs.0 == u64::MAX) {
            Some(Self::NAN)
        } else {
            self.0.checked_sub(rhs.0).map(Self)
        }
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for PartsPerMillion64 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Display for PartsPerMillion64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        f.write_str(buf.format(self.0))
    }
}

#[cfg(feature = "storage")]
impl Formattable for PartsPerMillion64 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut value = Buffer::new();
        buf.extend_from_slice(value.format(self.0).as_bytes());
    }

    #[inline(always)]
    fn fmt_json(&self, buf: &mut Vec<u8>) {
        if unlikely(self.0 == u64::MAX) {
            buf.extend_from_slice(b"null");
        } else {
            self.write_to(buf);
        }
    }
}
