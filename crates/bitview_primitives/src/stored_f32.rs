use std::{
    cmp::Ordering,
    f32,
    fmt::{Display, Formatter, Result},
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

use brk_types::{CheckedSub, Dollars};
use derive_more::Deref;
use ryu::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Close, StoredF64, StoredU32};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex};

/// Stored 32-bit floating point value
#[derive(Debug, Deref, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct StoredF32(f32);

impl StoredF32 {
    pub const NAN: Self = StoredF32(f32::NAN);
}

impl From<f32> for StoredF32 {
    #[inline]
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl From<f64> for StoredF32 {
    #[inline]
    fn from(value: f64) -> Self {
        debug_assert!(value.is_nan() || (f32::MIN as f64..=f32::MAX as f64).contains(&value));
        Self(value as f32)
    }
}

impl From<StoredF32> for f64 {
    #[inline]
    fn from(value: StoredF32) -> Self {
        value.0 as f64
    }
}

impl From<StoredF64> for StoredF32 {
    #[inline]
    fn from(value: StoredF64) -> Self {
        Self(*value as f32)
    }
}

impl From<usize> for StoredF32 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as f32)
    }
}

impl From<u8> for StoredF32 {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value as f32)
    }
}

impl From<StoredU32> for StoredF32 {
    #[inline]
    fn from(value: StoredU32) -> Self {
        Self(f32::from(value))
    }
}

impl CheckedSub<StoredF32> for StoredF32 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        Some(Self(self.0 - rhs.0))
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<StoredF32> for StoredF32 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl CheckedSub<usize> for StoredF32 {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        Some(Self(self.0 - rhs as f32))
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<usize> for StoredF32 {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Div<usize> for StoredF32 {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        if rhs == 0 {
            Self::NAN
        } else {
            Self(self.0 / rhs as f32)
        }
    }
}

impl Div<StoredU32> for StoredF32 {
    type Output = Self;
    fn div(self, rhs: StoredU32) -> Self::Output {
        let rhs = f32::from(rhs);
        if rhs == 0.0 {
            Self::NAN
        } else {
            Self(self.0 / rhs)
        }
    }
}

impl Add for StoredF32 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for StoredF32 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl SubAssign for StoredF32 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl From<StoredF32> for f32 {
    #[inline]
    fn from(value: StoredF32) -> Self {
        value.0
    }
}

impl From<Dollars> for StoredF32 {
    #[inline]
    fn from(value: Dollars) -> Self {
        StoredF32::from(f64::from(value))
    }
}

impl From<Close<Dollars>> for StoredF32 {
    #[inline]
    fn from(value: Close<Dollars>) -> Self {
        Self::from(*value)
    }
}

impl Div<Dollars> for StoredF32 {
    type Output = Self;
    fn div(self, rhs: Dollars) -> Self::Output {
        let rhs = *rhs;
        if rhs == 0.0 {
            Self::NAN
        } else {
            Self::from(self.0 as f64 / rhs)
        }
    }
}

impl Div<StoredF32> for StoredF32 {
    type Output = Self;
    fn div(self, rhs: StoredF32) -> Self::Output {
        if rhs.0 == 0.0 {
            Self::NAN
        } else {
            Self::from(self.0 / rhs.0)
        }
    }
}

impl Mul<usize> for StoredF32 {
    type Output = Self;
    fn mul(self, rhs: usize) -> Self::Output {
        Self(self.0 * rhs as f32)
    }
}

impl Mul<StoredF32> for StoredF32 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Mul<StoredF32> for usize {
    type Output = StoredF32;
    fn mul(self, rhs: StoredF32) -> Self::Output {
        StoredF32(self as f32 * rhs.0)
    }
}

impl Sub<StoredF32> for StoredF32 {
    type Output = Self;
    fn sub(self, rhs: StoredF32) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Neg for StoredF32 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl PartialEq for StoredF32 {
    fn eq(&self, other: &Self) -> bool {
        match (self.0.is_nan(), other.0.is_nan()) {
            (true, true) => true,
            (true, false) => false,
            (false, true) => false,
            (false, false) => self.0 == other.0,
        }
    }
}

impl Eq for StoredF32 {}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl PartialOrd for StoredF32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for StoredF32 {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.0.is_nan(), other.0.is_nan()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => self.0.partial_cmp(&other.0).unwrap(),
        }
    }
}

#[cfg(feature = "storage")]
impl PrintableIndex for StoredF32 {
    fn to_string() -> &'static str {
        "f32"
    }
    fn to_possible_strings() -> &'static [&'static str] {
        &["f32"]
    }
}

impl Sum for StoredF32 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|v| v.0).sum::<f32>())
    }
}

impl Display for StoredF32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for StoredF32 {
    const UNDEFINED: bool = true;

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

impl Mul<StoredF32> for Dollars {
    type Output = Self;
    fn mul(self, rhs: StoredF32) -> Self::Output {
        self * *rhs as f64
    }
}
