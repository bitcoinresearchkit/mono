use std::ops::{Add, AddAssign, Div};

use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use vecdb::{CheckedSub, Formattable, Pco, PrintableIndex};

#[derive(
    Default,
    Debug,
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
pub struct StoredU8(u8);

impl StoredU8 {
    pub const ZERO: Self = Self(0);

    pub fn new(counter: u8) -> Self {
        Self(counter)
    }
}

impl From<u8> for StoredU8 {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<usize> for StoredU8 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u8)
    }
}

impl CheckedSub<StoredU8> for StoredU8 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

impl Div<usize> for StoredU8 {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs as u8)
    }
}

impl Add for StoredU8 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for StoredU8 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl From<f64> for StoredU8 {
    #[inline]
    fn from(value: f64) -> Self {
        let value = value.max(0.0);
        debug_assert!(value <= u8::MAX as f64);
        Self(value as u8)
    }
}

impl From<StoredU8> for f64 {
    #[inline]
    fn from(value: StoredU8) -> Self {
        value.0 as f64
    }
}

impl Add<usize> for StoredU8 {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0.checked_add(rhs as u8).unwrap())
    }
}

impl From<StoredU8> for usize {
    #[inline]
    fn from(value: StoredU8) -> Self {
        value.0 as usize
    }
}

impl PrintableIndex for StoredU8 {
    fn to_string() -> &'static str {
        "u8"
    }

    fn to_possible_strings() -> &'static [&'static str] {
        &["u8"]
    }
}

impl std::fmt::Display for StoredU8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = itoa::Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

impl Formattable for StoredU8 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = itoa::Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
