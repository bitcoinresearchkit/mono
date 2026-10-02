use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Div},
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
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct StoredI16(i16);

impl From<i16> for StoredI16 {
    #[inline]
    fn from(value: i16) -> Self {
        Self(value)
    }
}

impl From<usize> for StoredI16 {
    #[inline]
    fn from(value: usize) -> Self {
        debug_assert!(value <= i16::MAX as usize);
        Self(value as i16)
    }
}

impl CheckedSub<StoredI16> for StoredI16 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<StoredI16> for StoredI16 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Div<usize> for StoredI16 {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs as i16)
    }
}

impl Add for StoredI16 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for StoredI16 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl From<f64> for StoredI16 {
    #[inline]
    fn from(value: f64) -> Self {
        debug_assert!(value >= 0.0 && value <= i16::MAX as f64);
        Self(value as i16)
    }
}

impl From<StoredI16> for f64 {
    #[inline]
    fn from(value: StoredI16) -> Self {
        value.0 as f64
    }
}

impl From<StoredI16> for usize {
    #[inline]
    fn from(value: StoredI16) -> Self {
        value.0 as usize
    }
}

#[cfg(feature = "storage")]
impl PrintableIndex for StoredI16 {
    fn to_string() -> &'static str {
        "i16"
    }
    fn to_possible_strings() -> &'static [&'static str] {
        &["i16"]
    }
}

impl Display for StoredI16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for StoredI16 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
