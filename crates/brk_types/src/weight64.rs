use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div, Sub, SubAssign},
};

use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use vecdb::{CheckedSub, Formattable, Pco};

use crate::{VSize, Weight};

/// Weight in weight units with enough range for cumulative and rolling totals.
#[derive(
    Debug,
    Default,
    Deref,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Pco,
    JsonSchema,
)]
pub struct Weight64(u64);

impl Weight64 {
    pub const ZERO: Self = Self(0);

    #[inline]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl From<Weight> for Weight64 {
    #[inline]
    fn from(value: Weight) -> Self {
        Self(u64::from(value))
    }
}

impl From<VSize> for Weight64 {
    #[inline]
    fn from(value: VSize) -> Self {
        debug_assert!(*value <= u64::MAX / 4);
        Self(*value * 4)
    }
}

impl From<u64> for Weight64 {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<usize> for Weight64 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}

impl From<f64> for Weight64 {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value.max(0.0) as u64)
    }
}

impl From<Weight64> for f64 {
    #[inline]
    fn from(value: Weight64) -> Self {
        value.0 as f64
    }
}

impl From<Weight64> for u64 {
    #[inline]
    fn from(value: Weight64) -> Self {
        value.0
    }
}

impl Add for Weight64 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Weight64 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sum for Weight64 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|value| value.0).sum())
    }
}

impl Sub for Weight64 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Weight64 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Div<usize> for Weight64 {
    type Output = Self;

    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs as u64)
    }
}

impl Div for Weight64 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl CheckedSub for Weight64 {
    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

impl std::fmt::Display for Weight64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = itoa::Buffer::new();
        f.write_str(buffer.format(self.0))
    }
}

impl Formattable for Weight64 {
    #[inline(always)]
    fn write_to(&self, buffer: &mut Vec<u8>) {
        let mut value = itoa::Buffer::new();
        buffer.extend_from_slice(value.format(self.0).as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holds_weight_totals_above_u32() {
        let total = Weight64::new(u64::from(u32::MAX) + 1);
        assert_eq!(u64::from(total), u64::from(u32::MAX) + 1);
    }
}
