use std::fmt::{Display, Formatter, Result as FmtResult};

use derive_more::Deref;
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::unlikely;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// A ratio in [0, 1], floored at scale u32::MAX - 1.
/// Zero and one are exact; finite quantization error is less than 1 / SCALE
/// apart from floating-point arithmetic error. u32::MAX represents undefined.
/// Non-finite inputs become undefined. Finite inputs must lie in [0, 1]; this
/// precondition is checked only in debug builds. Keep cumulative state unrounded.
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
pub struct BoundedRatio(u32);

impl BoundedRatio {
    pub const SCALE: u32 = u32::MAX - 1;
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(Self::SCALE);
    pub const NAN: Self = Self(u32::MAX);

    /// Derive the complement without a float round-trip or a second stored source.
    pub const fn complement(self) -> Self {
        if self.is_nan() {
            Self::NAN
        } else {
            Self(Self::SCALE - self.0)
        }
    }

    /// Restore raw encoded bits, including the undefined sentinel.
    pub const fn from_raw(value: u32) -> Self {
        Self(value)
    }

    pub const fn inner(self) -> u32 {
        self.0
    }

    pub const fn is_nan(self) -> bool {
        self.0 == u32::MAX
    }
}

impl From<f64> for BoundedRatio {
    #[inline]
    fn from(value: f64) -> Self {
        if unlikely(!value.is_finite()) {
            Self::NAN
        } else {
            debug_assert!(
                (0.0..=1.0).contains(&value),
                "BoundedRatio out of range: {value}"
            );
            Self((value * Self::SCALE as f64).floor() as u32)
        }
    }
}

impl From<BoundedRatio> for f64 {
    #[inline]
    fn from(value: BoundedRatio) -> Self {
        if unlikely(value.is_nan()) {
            f64::NAN
        } else {
            value.0 as f64 / BoundedRatio::SCALE as f64
        }
    }
}

impl From<BoundedRatio> for f32 {
    #[inline]
    fn from(value: BoundedRatio) -> Self {
        f64::from(value) as f32
    }
}

impl Display for BoundedRatio {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut buf = Buffer::new();
        f.write_str(buf.format(self.0))
    }
}

#[cfg(feature = "storage")]
impl Formattable for BoundedRatio {
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
