use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Div, Sub, SubAssign},
};

use derive_more::Deref;
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::CheckedSub;

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex};

/// Fixed-size 64-bit signed integer optimized for on-disk storage
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
pub struct StoredI64(i64);

impl From<i64> for StoredI64 {
    #[inline]
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<StoredI64> for i64 {
    #[inline]
    fn from(value: StoredI64) -> Self {
        value.0
    }
}

impl From<usize> for StoredI64 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as i64)
    }
}

impl From<StoredI64> for usize {
    #[inline]
    fn from(value: StoredI64) -> Self {
        value.0 as usize
    }
}

impl CheckedSub<StoredI64> for StoredI64 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<StoredI64> for StoredI64 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Div<usize> for StoredI64 {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs as i64)
    }
}

impl Add for StoredI64 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for StoredI64 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sub for StoredI64 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for StoredI64 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl From<f64> for StoredI64 {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value as i64)
    }
}

impl From<StoredI64> for f64 {
    #[inline]
    fn from(value: StoredI64) -> Self {
        value.0 as f64
    }
}

impl StoredI64 {
    fn index_name() -> &'static str {
        "i64"
    }
    fn index_aliases() -> &'static [&'static str] {
        &["i64"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for StoredI64 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

impl Display for StoredI64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for StoredI64 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
