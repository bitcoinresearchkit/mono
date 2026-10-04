use std::{
    fmt::{Display, Formatter, Result},
    iter::Sum,
    ops::{Add, AddAssign, Div, Sub, SubAssign},
};

use brk_types::CheckedSub;
use derive_more::Deref;
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Fixed-size 64-bit unsigned integer optimized for on-disk storage
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
    JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct StoredU64(u64);

impl StoredU64 {
    pub const ZERO: Self = Self(0);

    pub fn new(counter: u64) -> Self {
        Self(counter)
    }
}

impl From<u64> for StoredU64 {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<StoredU64> for u64 {
    #[inline]
    fn from(value: StoredU64) -> Self {
        value.0
    }
}

impl From<StoredU64> for usize {
    #[inline]
    fn from(value: StoredU64) -> Self {
        value.0 as usize
    }
}

impl From<usize> for StoredU64 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}

impl CheckedSub<StoredU64> for StoredU64 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<StoredU64> for StoredU64 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Div<usize> for StoredU64 {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs as u64)
    }
}

impl Add for StoredU64 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for StoredU64 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sum for StoredU64 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|v| v.0).sum())
    }
}

impl Sub for StoredU64 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for StoredU64 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl From<f64> for StoredU64 {
    #[inline]
    fn from(value: f64) -> Self {
        let value = value.max(0.0);
        Self(value as u64)
    }
}

impl From<StoredU64> for f64 {
    #[inline]
    fn from(value: StoredU64) -> Self {
        value.0 as f64
    }
}

impl Display for StoredU64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for StoredU64 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
