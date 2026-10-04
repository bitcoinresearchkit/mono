use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Div, Sub},
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

/// Unsigned parts per million stored as u32.
/// One unit is 0.000001. Range: 0–4,294.967294.
/// Use for precise bounded ratios and percentages.
/// `u32::MAX` is reserved as a NaN sentinel.
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
pub struct PartsPerMillion32(u32);

impl PartsPerMillion32 {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1_000_000);
    pub const NAN: Self = Self(u32::MAX);

    #[inline]
    pub const fn new(value: u32) -> Self {
        debug_assert!(value != u32::MAX, "u32::MAX is reserved as NaN sentinel");
        Self(value)
    }

    #[inline]
    fn is_nan(self) -> bool {
        self.0 == u32::MAX
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

impl From<usize> for PartsPerMillion32 {
    #[inline]
    fn from(value: usize) -> Self {
        debug_assert!(
            value < u32::MAX as usize,
            "usize out of PartsPerMillion32 range: {value}"
        );
        Self(value as u32)
    }
}

impl From<u32> for PartsPerMillion32 {
    #[inline]
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl From<PartsPerMillion32> for u32 {
    #[inline]
    fn from(value: PartsPerMillion32) -> Self {
        value.0
    }
}

impl From<f32> for PartsPerMillion32 {
    #[inline]
    fn from(value: f32) -> Self {
        if unlikely(!value.is_finite()) {
            return Self::NAN;
        }
        Self::from(value as f64)
    }
}

impl From<f64> for PartsPerMillion32 {
    #[inline]
    fn from(value: f64) -> Self {
        if unlikely(!value.is_finite()) {
            return Self::NAN;
        }
        let scaled = (value * 1_000_000.0)
            .round()
            .clamp(0.0, u32::MAX as f64 - 1.0);
        Self(scaled as u32)
    }
}

impl From<PartsPerMillion32> for f64 {
    #[inline]
    fn from(value: PartsPerMillion32) -> Self {
        if unlikely(value.0 == u32::MAX) {
            f64::NAN
        } else {
            value.0 as f64 / 1_000_000.0
        }
    }
}

impl From<PartsPerMillion32> for f32 {
    #[inline]
    fn from(value: PartsPerMillion32) -> Self {
        value.to_f32()
    }
}

impl Add for PartsPerMillion32 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        if unlikely(self.0 == u32::MAX || rhs.0 == u32::MAX) {
            Self::NAN
        } else {
            Self(self.0 + rhs.0)
        }
    }
}

impl Sub for PartsPerMillion32 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        if unlikely(self.0 == u32::MAX || rhs.0 == u32::MAX) {
            Self::NAN
        } else {
            Self(self.0 - rhs.0)
        }
    }
}

impl AddAssign for PartsPerMillion32 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Div<usize> for PartsPerMillion32 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: usize) -> Self::Output {
        if unlikely(self.0 == u32::MAX) {
            Self::NAN
        } else {
            debug_assert!(rhs <= u32::MAX as usize, "divisor out of u32 range: {rhs}");
            Self(self.0 / rhs as u32)
        }
    }
}

impl CheckedSub for PartsPerMillion32 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        if unlikely(self.0 == u32::MAX || rhs.0 == u32::MAX) {
            Some(Self::NAN)
        } else {
            self.0.checked_sub(rhs.0).map(Self)
        }
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for PartsPerMillion32 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Display for PartsPerMillion32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        f.write_str(buf.format(self.0))
    }
}

#[cfg(feature = "storage")]
impl Formattable for PartsPerMillion32 {
    const UNDEFINED: bool = true;

    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut value = Buffer::new();
        buf.extend_from_slice(value.format(self.0).as_bytes());
    }

    #[inline(always)]
    fn fmt_json(&self, buf: &mut Vec<u8>) {
        if unlikely(self.0 == u32::MAX) {
            buf.extend_from_slice(b"null");
        } else {
            self.write_to(buf);
        }
    }

    /// Undefined: an empty CSV cell (JSON writes `null`).
    #[inline(always)]
    fn fmt_csv(&self, f: &mut String) -> std::fmt::Result {
        if !unlikely(self.0 == u32::MAX) {
            self.fmt_into(f);
        }
        Ok(())
    }
}
