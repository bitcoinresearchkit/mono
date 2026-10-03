use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use itoa::Buffer;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Dollars;
use crate::CheckedSub;

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Signed cents (i64) - for values that can be negative.
/// Used for profit/loss calculations, deltas, etc.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct CentsSigned(i64);

impl CentsSigned {
    #[inline]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn inner(self) -> i64 {
        self.0
    }

    #[inline]
    pub(crate) fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    fn to_dollars(self) -> Dollars {
        Dollars::from(self.0 as f64 / 100.0)
    }
}

impl From<Dollars> for CentsSigned {
    #[inline]
    fn from(value: Dollars) -> Self {
        Self((*value * 100.0).round() as i64)
    }
}

impl From<CentsSigned> for Dollars {
    #[inline]
    fn from(value: CentsSigned) -> Self {
        value.to_dollars()
    }
}

impl From<CentsSigned> for f64 {
    #[inline]
    fn from(value: CentsSigned) -> Self {
        value.0 as f64
    }
}

impl From<f64> for CentsSigned {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value as i64)
    }
}

impl From<i64> for CentsSigned {
    #[inline]
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<u64> for CentsSigned {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value as i64)
    }
}

impl From<CentsSigned> for usize {
    #[inline]
    fn from(value: CentsSigned) -> Self {
        debug_assert!(value.0 >= 0, "Cannot convert negative CentsSigned to usize");
        value.0 as usize
    }
}

impl From<usize> for CentsSigned {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as i64)
    }
}

impl From<CentsSigned> for i64 {
    #[inline]
    fn from(value: CentsSigned) -> Self {
        value.0
    }
}

impl From<CentsSigned> for u64 {
    #[inline]
    fn from(value: CentsSigned) -> Self {
        debug_assert!(value.0 >= 0, "Cannot convert negative CentsSigned to u64");
        value.0 as u64
    }
}

impl Add for CentsSigned {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for CentsSigned {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for CentsSigned {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for CentsSigned {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Div<CentsSigned> for CentsSigned {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Div<usize> for CentsSigned {
    type Output = Self;
    #[inline]
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs as i64)
    }
}

impl From<CentsSigned> for i128 {
    #[inline]
    fn from(value: CentsSigned) -> Self {
        value.0 as i128
    }
}

impl From<i128> for CentsSigned {
    #[inline]
    fn from(value: i128) -> Self {
        debug_assert!(
            value >= i64::MIN as i128 && value <= i64::MAX as i128,
            "i128 overflow to CentsSigned"
        );
        Self(value as i64)
    }
}

impl From<u128> for CentsSigned {
    #[inline]
    fn from(value: u128) -> Self {
        debug_assert!(value <= i64::MAX as u128, "u128 overflow to CentsSigned");
        Self(value as i64)
    }
}

impl From<CentsSigned> for u128 {
    #[inline]
    fn from(value: CentsSigned) -> Self {
        debug_assert!(value.0 >= 0, "Cannot convert negative CentsSigned to u128");
        value.0 as u128
    }
}

impl Mul<CentsSigned> for CentsSigned {
    type Output = CentsSigned;
    #[inline]
    fn mul(self, rhs: CentsSigned) -> Self::Output {
        Self(self.0.checked_mul(rhs.0).expect("CentsSigned overflow"))
    }
}

impl Mul<i64> for CentsSigned {
    type Output = CentsSigned;
    #[inline]
    fn mul(self, rhs: i64) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Mul<usize> for CentsSigned {
    type Output = CentsSigned;
    #[inline]
    fn mul(self, rhs: usize) -> Self::Output {
        Self(self.0 * rhs as i64)
    }
}

impl CheckedSub for CentsSigned {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.checked_sub(rhs)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for CentsSigned {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Display for CentsSigned {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for CentsSigned {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
