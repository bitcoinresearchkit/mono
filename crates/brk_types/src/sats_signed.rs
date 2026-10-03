use std::{
    fmt::{Display, Formatter, Result},
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

use derive_more::Deref;
use itoa::Buffer;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Bitcoin, Sats};
use crate::CheckedSub;

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Signed satoshis (i64) - for values that can be negative.
/// Used for changes, deltas, profit/loss calculations, etc.
#[derive(
    Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy, Deref, Default, Serialize, Deserialize,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct SatsSigned(i64);

impl SatsSigned {
    const ZERO: Self = Self(0);

    #[inline]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn inner(self) -> i64 {
        self.0
    }
}

impl Add for SatsSigned {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for SatsSigned {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl AddAssign for SatsSigned {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for SatsSigned {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Neg for SatsSigned {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl CheckedSub for SatsSigned {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for SatsSigned {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
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

impl Div<usize> for SatsSigned {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        if rhs == 0 {
            Self::ZERO
        } else {
            Self(self.0 / rhs as i64)
        }
    }
}

impl Sum for SatsSigned {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let sats: i64 = iter.map(|s| s.0).sum();
        Self(sats)
    }
}

impl From<i64> for SatsSigned {
    #[inline]
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<SatsSigned> for i64 {
    #[inline]
    fn from(value: SatsSigned) -> Self {
        value.0
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
        Self::from(Sats::from(value))
    }
}

impl From<SatsSigned> for Bitcoin {
    #[inline]
    fn from(value: SatsSigned) -> Self {
        Self::from(value.0 as f64 / Sats::ONE_BTC_U64 as f64)
    }
}

impl Display for SatsSigned {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for SatsSigned {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
