use std::ops::Mul;

use derive_more::{Add, AddAssign, Deref, From, Into, Neg, Sub, SubAssign, Sum};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Bitcoin, Sats};
use crate::CheckedSub;

#[cfg(feature = "storage")]
use vecdb::Pco;

/// Signed satoshis (i64) - for values that can be negative.
/// Used for changes, deltas, profit/loss calculations, etc.
#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    Deref,
    Default,
    Serialize,
    Deserialize,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Neg,
    Sum,
    From,
    Into,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct SatsSigned(i64);

impl SatsSigned {
    #[inline]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn inner(self) -> i64 {
        self.0
    }
}

impl CheckedSub for SatsSigned {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

impl Mul<i64> for SatsSigned {
    type Output = Self;
    fn mul(self, rhs: i64) -> Self::Output {
        Self(self.0.checked_mul(rhs).expect("SatsSigned overflow"))
    }
}

impl Mul<usize> for SatsSigned {
    type Output = Self;
    fn mul(self, rhs: usize) -> Self::Output {
        Self(self.0.checked_mul(rhs as i64).expect("SatsSigned overflow"))
    }
}

impl From<usize> for SatsSigned {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as i64)
    }
}

impl From<f32> for SatsSigned {
    #[inline]
    fn from(value: f32) -> Self {
        Self(value.round() as i64)
    }
}

impl From<SatsSigned> for f32 {
    #[inline]
    fn from(value: SatsSigned) -> Self {
        value.0 as f32
    }
}

impl From<f64> for SatsSigned {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value.round() as i64)
    }
}

impl From<SatsSigned> for f64 {
    #[inline]
    fn from(value: SatsSigned) -> Self {
        value.0 as f64
    }
}

impl From<Sats> for SatsSigned {
    #[inline]
    fn from(value: Sats) -> Self {
        Self(*value as i64)
    }
}

impl From<Bitcoin> for SatsSigned {
    #[inline]
    fn from(value: Bitcoin) -> Self {
        Self((f64::from(value) * Sats::ONE_BTC_U64 as f64).round() as i64)
    }
}

impl From<SatsSigned> for Bitcoin {
    #[inline]
    fn from(value: SatsSigned) -> Self {
        Self::from(value.0 as f64 / Sats::ONE_BTC_U64 as f64)
    }
}

crate::impl_int_format!(SatsSigned);
