use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Div},
};

use brk_types::CheckedSub;
use derive_more::Deref;
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
pub struct StoredI8(i8);

impl StoredI8 {
    pub fn new(v: i8) -> Self {
        Self(v)
    }
}

impl From<i8> for StoredI8 {
    #[inline]
    fn from(value: i8) -> Self {
        Self(value)
    }
}

impl From<usize> for StoredI8 {
    #[inline]
    fn from(value: usize) -> Self {
        debug_assert!(value <= i8::MAX as usize);
        Self(value as i8)
    }
}

impl CheckedSub<StoredI8> for StoredI8 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<StoredI8> for StoredI8 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Div<usize> for StoredI8 {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs as i8)
    }
}

impl Add for StoredI8 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for StoredI8 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl From<f64> for StoredI8 {
    #[inline]
    fn from(value: f64) -> Self {
        debug_assert!(value >= i8::MIN as f64 && value <= i8::MAX as f64);
        Self(value as i8)
    }
}

impl From<StoredI8> for f64 {
    #[inline]
    fn from(value: StoredI8) -> Self {
        value.0 as f64
    }
}

impl From<StoredI8> for usize {
    #[inline]
    fn from(value: StoredI8) -> Self {
        value.0 as usize
    }
}

#[cfg(feature = "storage")]
impl PrintableIndex for StoredI8 {
    fn to_string() -> &'static str {
        "i8"
    }
    fn to_possible_strings() -> &'static [&'static str] {
        &["i8"]
    }
}

impl Display for StoredI8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for StoredI8 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
