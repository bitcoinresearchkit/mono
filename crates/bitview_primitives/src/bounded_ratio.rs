use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::unlikely;

#[cfg(feature = "storage")]
use vecdb::Pco;

/// A ratio between 0 and 1.
///
/// Floored at scale u32::MAX - 1. Zero and one are exact; finite quantization error is less than 1 / SCALE
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

brk_types::impl_int_format!(BoundedRatio, nullable);
