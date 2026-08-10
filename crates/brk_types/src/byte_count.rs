use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div, Sub, SubAssign},
};

use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use vecdb::{CheckedSub, Formattable, Pco};

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
pub struct Bytes(u64);

impl Bytes {
    #[inline]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl From<u64> for Bytes {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<u32> for Bytes {
    #[inline]
    fn from(value: u32) -> Self {
        Self(u64::from(value))
    }
}

impl From<usize> for Bytes {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}

impl From<Bytes> for u64 {
    #[inline]
    fn from(value: Bytes) -> Self {
        value.0
    }
}

impl From<f64> for Bytes {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value.max(0.0) as u64)
    }
}

impl From<Bytes> for f64 {
    #[inline]
    fn from(value: Bytes) -> Self {
        value.0 as f64
    }
}

impl Add for Bytes {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Bytes {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Bytes {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Bytes {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Div<usize> for Bytes {
    type Output = Self;

    #[inline]
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs as u64)
    }
}

impl Sum for Bytes {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|value| value.0).sum())
    }
}

impl CheckedSub for Bytes {
    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

impl std::fmt::Display for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = itoa::Buffer::new();
        f.write_str(buffer.format(self.0))
    }
}

impl Formattable for Bytes {
    #[inline(always)]
    fn write_to(&self, buffer: &mut Vec<u8>) {
        let mut formatted = itoa::Buffer::new();
        buffer.extend_from_slice(formatted.format(self.0).as_bytes());
    }
}
