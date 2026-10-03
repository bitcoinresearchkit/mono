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

/// Unsigned basis points: 10,000 represents the ratio 1.
/// Maximum finite ratio: 429,496.7294. u32::MAX represents undefined.
/// Finite input range is a debug-checked precondition, not a saturation policy.
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
pub struct BasisPoints32(u32);

impl BasisPoints32 {
    const SCALE: u32 = 10_000;
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(u32::MAX - 1);
    pub const NAN: Self = Self(u32::MAX);

    pub const fn is_nan(self) -> bool {
        self.0 == u32::MAX
    }
}

impl From<f64> for BasisPoints32 {
    #[inline]
    fn from(value: f64) -> Self {
        if unlikely(!value.is_finite()) {
            return Self::NAN;
        }
        let raw = (value * Self::SCALE as f64).floor();
        debug_assert!(
            value >= 0.0 && raw <= Self::MAX.0 as f64,
            "BasisPoints32 out of range: {value}"
        );
        Self(raw as u32)
    }
}

impl From<usize> for BasisPoints32 {
    #[inline]
    fn from(raw: usize) -> Self {
        debug_assert!(
            raw < u32::MAX as usize,
            "BasisPoints32 raw value out of range"
        );
        Self(raw as u32)
    }
}

impl Add for BasisPoints32 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        if unlikely(self.is_nan() || rhs.is_nan()) {
            Self::NAN
        } else {
            let raw = self.0 as u64 + rhs.0 as u64;
            debug_assert!(raw < u32::MAX as u64, "BasisPoints32 sum out of range");
            Self(raw as u32)
        }
    }
}

impl AddAssign for BasisPoints32 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Div<usize> for BasisPoints32 {
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

impl CheckedSub for BasisPoints32 {
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
impl VecdbCheckedSub for BasisPoints32 {
    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl From<PartsPerMillion64> for BasisPoints32 {
    #[inline]
    fn from(value: PartsPerMillion64) -> Self {
        if unlikely(value.is_nan()) {
            return Self::NAN;
        }
        let raw = value.inner() / 100;
        debug_assert!(raw <= Self::MAX.0 as u64, "PPM out of BasisPoints32 range");
        Self(raw as u32)
    }
}

impl From<BasisPoints32> for f64 {
    #[inline]
    fn from(value: BasisPoints32) -> Self {
        if unlikely(value.is_nan()) {
            f64::NAN
        } else {
            value.0 as f64 / BasisPoints32::SCALE as f64
        }
    }
}

impl From<BasisPoints32> for f32 {
    #[inline]
    fn from(value: BasisPoints32) -> Self {
        f64::from(value) as f32
    }
}

impl From<BasisPoints32> for StoredF32 {
    #[inline]
    fn from(value: BasisPoints32) -> Self {
        Self::from(f32::from(value))
    }
}

impl Display for BasisPoints32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut buf = Buffer::new();
        f.write_str(buf.format(self.0))
    }
}

#[cfg(feature = "storage")]
impl Formattable for BasisPoints32 {
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
}
