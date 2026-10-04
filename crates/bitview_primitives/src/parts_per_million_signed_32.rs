use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Div, Sub, SubAssign},
};

use brk_types::CheckedSub;
use derive_more::Deref;
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{StoredF32, unlikely};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Signed parts per million stored as i32.
/// One unit is 0.000001. Range: -2,147.483647 to +2,147.483647.
/// Use for precise bounded signed ratios and percentages.
/// `i32::MIN` is reserved as a NaN sentinel.
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
pub struct PartsPerMillionSigned32(i32);

impl PartsPerMillionSigned32 {
    pub const ZERO: Self = Self(0);
    pub const NAN: Self = Self(i32::MIN);

    #[inline]
    const fn new(value: i32) -> Self {
        debug_assert!(value != i32::MIN, "i32::MIN is reserved as NaN sentinel");
        Self(value)
    }

    #[inline]
    fn is_nan(self) -> bool {
        self.0 == i32::MIN
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

impl From<usize> for PartsPerMillionSigned32 {
    #[inline]
    fn from(value: usize) -> Self {
        debug_assert!(
            value <= i32::MAX as usize,
            "usize out of PartsPerMillionSigned32 range: {value}"
        );
        Self(value as i32)
    }
}

impl From<i32> for PartsPerMillionSigned32 {
    #[inline]
    fn from(value: i32) -> Self {
        Self::new(value)
    }
}

impl From<PartsPerMillionSigned32> for i32 {
    #[inline]
    fn from(value: PartsPerMillionSigned32) -> Self {
        value.0
    }
}

impl From<f64> for PartsPerMillionSigned32 {
    #[inline]
    fn from(value: f64) -> Self {
        if unlikely(!value.is_finite()) {
            return Self::NAN;
        }
        let scaled = (value * 1_000_000.0)
            .round()
            .clamp(i32::MIN as f64 + 1.0, i32::MAX as f64);
        Self(scaled as i32)
    }
}

impl From<f32> for PartsPerMillionSigned32 {
    #[inline]
    fn from(value: f32) -> Self {
        Self::from(value as f64)
    }
}

impl From<StoredF32> for PartsPerMillionSigned32 {
    #[inline]
    fn from(value: StoredF32) -> Self {
        Self::from(*value)
    }
}

impl From<PartsPerMillionSigned32> for f64 {
    #[inline]
    fn from(value: PartsPerMillionSigned32) -> Self {
        if unlikely(value.0 == i32::MIN) {
            f64::NAN
        } else {
            value.0 as f64 / 1_000_000.0
        }
    }
}

impl From<PartsPerMillionSigned32> for f32 {
    #[inline]
    fn from(value: PartsPerMillionSigned32) -> Self {
        value.to_f32()
    }
}

impl From<PartsPerMillionSigned32> for StoredF32 {
    #[inline]
    fn from(value: PartsPerMillionSigned32) -> Self {
        StoredF32::from(value.to_f32())
    }
}

impl Add for PartsPerMillionSigned32 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        if unlikely(self.0 == i32::MIN || rhs.0 == i32::MIN) {
            Self::NAN
        } else {
            Self(self.0 + rhs.0)
        }
    }
}

impl AddAssign for PartsPerMillionSigned32 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for PartsPerMillionSigned32 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        if unlikely(self.0 == i32::MIN || rhs.0 == i32::MIN) {
            Self::NAN
        } else {
            Self(self.0 - rhs.0)
        }
    }
}

impl SubAssign for PartsPerMillionSigned32 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Div<usize> for PartsPerMillionSigned32 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: usize) -> Self::Output {
        if unlikely(self.0 == i32::MIN) {
            Self::NAN
        } else {
            debug_assert!(rhs <= i32::MAX as usize, "divisor out of i32 range: {rhs}");
            Self(self.0 / rhs as i32)
        }
    }
}

impl CheckedSub for PartsPerMillionSigned32 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        if unlikely(self.0 == i32::MIN || rhs.0 == i32::MIN) {
            Some(Self::NAN)
        } else {
            self.0.checked_sub(rhs.0).map(Self)
        }
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for PartsPerMillionSigned32 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Display for PartsPerMillionSigned32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        f.write_str(buf.format(self.0))
    }
}

#[cfg(feature = "storage")]
impl Formattable for PartsPerMillionSigned32 {
    const UNDEFINED: bool = true;

    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut value = Buffer::new();
        buf.extend_from_slice(value.format(self.0).as_bytes());
    }

    #[inline(always)]
    fn fmt_json(&self, buf: &mut Vec<u8>) {
        if unlikely(self.0 == i32::MIN) {
            buf.extend_from_slice(b"null");
        } else {
            self.write_to(buf);
        }
    }

    /// Undefined: an empty CSV cell (JSON writes `null`).
    #[inline(always)]
    fn fmt_csv(&self, f: &mut String) -> std::fmt::Result {
        if !unlikely(self.0 == i32::MIN) {
            self.fmt_into(f);
        }
        Ok(())
    }
}
