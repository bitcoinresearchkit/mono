use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Sub, SubAssign},
};

use brk_types::CheckedSub;
use derive_more::Deref;
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::unlikely;

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Signed parts per million stored as i64.
/// One unit is 0.000001. Range: -9,223,372,036,854.775807 to +9,223,372,036,854.775807.
/// Use for precise wide-range signed ratios and percentages.
/// `i64::MIN` is reserved as a NaN sentinel.
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
pub struct PartsPerMillionSigned64(i64);

impl PartsPerMillionSigned64 {
    const NAN: Self = Self(i64::MIN);

    #[inline]
    const fn new(value: i64) -> Self {
        debug_assert!(value != i64::MIN, "i64::MIN is reserved as NaN sentinel");
        Self(value)
    }

    #[inline]
    fn is_nan(self) -> bool {
        self.0 == i64::MIN
    }

    #[inline]
    fn to_f32(self) -> f32 {
        if unlikely(self.is_nan()) {
            f32::NAN
        } else {
            self.0 as f32 / 1_000_000.0
        }
    }
}

impl From<usize> for PartsPerMillionSigned64 {
    #[inline]
    fn from(value: usize) -> Self {
        debug_assert!(
            value <= i64::MAX as usize,
            "usize out of PartsPerMillionSigned64 range: {value}"
        );
        Self(value as i64)
    }
}

impl From<i64> for PartsPerMillionSigned64 {
    #[inline]
    fn from(value: i64) -> Self {
        Self::new(value)
    }
}

impl From<PartsPerMillionSigned64> for i64 {
    #[inline]
    fn from(value: PartsPerMillionSigned64) -> Self {
        value.0
    }
}

impl From<f64> for PartsPerMillionSigned64 {
    #[inline]
    fn from(value: f64) -> Self {
        if unlikely(!value.is_finite()) {
            return Self::NAN;
        }
        let scaled = (value * 1_000_000.0).round();
        if scaled <= i64::MIN as f64 {
            Self(i64::MIN + 1)
        } else if scaled >= i64::MAX as f64 {
            Self(i64::MAX)
        } else {
            Self(scaled as i64)
        }
    }
}

impl From<f32> for PartsPerMillionSigned64 {
    #[inline]
    fn from(value: f32) -> Self {
        Self::from(value as f64)
    }
}

impl From<PartsPerMillionSigned64> for f64 {
    #[inline]
    fn from(value: PartsPerMillionSigned64) -> Self {
        if unlikely(value.0 == i64::MIN) {
            f64::NAN
        } else {
            value.0 as f64 / 1_000_000.0
        }
    }
}

impl From<PartsPerMillionSigned64> for f32 {
    #[inline]
    fn from(value: PartsPerMillionSigned64) -> Self {
        value.to_f32()
    }
}

impl Add for PartsPerMillionSigned64 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        if unlikely(self.0 == i64::MIN || rhs.0 == i64::MIN) {
            Self::NAN
        } else {
            Self(self.0 + rhs.0)
        }
    }
}

impl AddAssign for PartsPerMillionSigned64 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for PartsPerMillionSigned64 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        if unlikely(self.0 == i64::MIN || rhs.0 == i64::MIN) {
            Self::NAN
        } else {
            Self(self.0 - rhs.0)
        }
    }
}

impl SubAssign for PartsPerMillionSigned64 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl CheckedSub for PartsPerMillionSigned64 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        if unlikely(self.0 == i64::MIN || rhs.0 == i64::MIN) {
            Some(Self::NAN)
        } else {
            self.0.checked_sub(rhs.0).map(Self)
        }
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for PartsPerMillionSigned64 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Display for PartsPerMillionSigned64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        f.write_str(buf.format(self.0))
    }
}

#[cfg(feature = "storage")]
impl Formattable for PartsPerMillionSigned64 {
    const UNDEFINED: bool = true;

    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut value = Buffer::new();
        buf.extend_from_slice(value.format(self.0).as_bytes());
    }

    #[inline(always)]
    fn fmt_json(&self, buf: &mut Vec<u8>) {
        if unlikely(self.0 == i64::MIN) {
            buf.extend_from_slice(b"null");
        } else {
            self.write_to(buf);
        }
    }

    /// Undefined: an empty CSV cell (JSON writes `null`).
    #[inline(always)]
    fn fmt_csv(&self, f: &mut String) -> std::fmt::Result {
        if !unlikely(self.0 == i64::MIN) {
            self.fmt_into(f);
        }
        Ok(())
    }
}
