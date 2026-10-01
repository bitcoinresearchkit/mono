use std::{
    cmp::Ordering,
    f64,
    fmt::{Display, Formatter, Result},
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use derive_more::Deref;
use ryu::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Bitcoin, Cents, CheckedSub, Dollars, Sats, StoredU64};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex};

/// Fixed-size 64-bit floating point value optimized for on-disk storage
#[derive(Debug, Deref, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct StoredF64(f64);

impl StoredF64 {
    pub const NAN: Self = Self(f64::NAN);
}

impl From<f64> for StoredF64 {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<f32> for StoredF64 {
    #[inline]
    fn from(value: f32) -> Self {
        Self(value as f64)
    }
}

impl From<u8> for StoredF64 {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value as f64)
    }
}

impl From<usize> for StoredF64 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as f64)
    }
}

impl From<StoredU64> for StoredF64 {
    #[inline]
    fn from(value: StoredU64) -> Self {
        Self(*value as f64)
    }
}

impl CheckedSub<StoredF64> for StoredF64 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        Some(Self(self.0 - rhs.0))
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<StoredF64> for StoredF64 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Mul<usize> for StoredF64 {
    type Output = Self;
    fn mul(self, rhs: usize) -> Self::Output {
        Self(self.0 * rhs as f64)
    }
}

impl Sub for StoredF64 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for StoredF64 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Mul<Dollars> for StoredF64 {
    type Output = Self;
    fn mul(self, rhs: Dollars) -> Self::Output {
        Self(self.0 * *rhs)
    }
}

impl Mul<Sats> for StoredF64 {
    type Output = Sats;
    #[inline]
    fn mul(self, rhs: Sats) -> Self::Output {
        rhs * self
    }
}

impl Mul<Cents> for StoredF64 {
    type Output = Cents;
    #[inline]
    fn mul(self, rhs: Cents) -> Self::Output {
        rhs * self
    }
}

impl Div<usize> for StoredF64 {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        if rhs == 0 {
            Self::NAN
        } else {
            Self(self.0 / rhs as f64)
        }
    }
}

impl Div for StoredF64 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.0 == 0.0 {
            Self::NAN
        } else {
            Self(self.0 / rhs.0)
        }
    }
}

impl Div<Dollars> for StoredF64 {
    type Output = Self;
    fn div(self, rhs: Dollars) -> Self::Output {
        let rhs = *rhs;
        if rhs == 0.0 {
            Self::NAN
        } else {
            Self(self.0 / rhs)
        }
    }
}

impl Add for StoredF64 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for StoredF64 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl SubAssign for StoredF64 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl From<StoredF64> for f64 {
    #[inline]
    fn from(value: StoredF64) -> Self {
        value.0
    }
}

impl From<StoredF64> for f32 {
    #[inline]
    fn from(value: StoredF64) -> Self {
        value.0 as f32
    }
}

impl From<Dollars> for StoredF64 {
    #[inline]
    fn from(value: Dollars) -> Self {
        Self(f64::from(value))
    }
}

impl CheckedSub<usize> for StoredF64 {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        Some(Self(self.0 - rhs as f64))
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<usize> for StoredF64 {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl PartialEq for StoredF64 {
    fn eq(&self, other: &Self) -> bool {
        match (self.0.is_nan(), other.0.is_nan()) {
            (true, true) => true,
            (true, false) => false,
            (false, true) => false,
            (false, false) => self.0 == other.0,
        }
    }
}

impl Eq for StoredF64 {}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl PartialOrd for StoredF64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for StoredF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.0.is_nan(), other.0.is_nan()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => self.0.partial_cmp(&other.0).unwrap(),
        }
    }
}

impl From<Bitcoin> for StoredF64 {
    #[inline]
    fn from(value: Bitcoin) -> Self {
        Self(f64::from(value))
    }
}

impl StoredF64 {
    fn index_name() -> &'static str {
        "f64"
    }
    fn index_aliases() -> &'static [&'static str] {
        &["f64"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for StoredF64 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

impl Sum for StoredF64 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|v| v.0).sum::<f64>())
    }
}

impl Div<Bitcoin> for StoredF64 {
    type Output = Self;
    fn div(self, rhs: Bitcoin) -> Self::Output {
        let rhs = f64::from(rhs);
        if rhs == 0.0 {
            Self::NAN
        } else {
            Self(self.0 / rhs)
        }
    }
}

impl Display for StoredF64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for StoredF64 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        if self.0.is_finite() {
            let mut b = Buffer::new();
            buf.extend_from_slice(b.format(self.0).as_bytes());
        }
    }

    #[inline(always)]
    fn fmt_json(&self, buf: &mut Vec<u8>) {
        if self.0.is_finite() {
            self.write_to(buf);
        } else {
            buf.extend_from_slice(b"null");
        }
    }
}
