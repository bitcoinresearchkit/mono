use std::{
    cmp::Ordering,
    f64,
    fmt::{Display, Formatter, Result},
    hash::{Hash, Hasher},
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

use derive_more::Deref;
use ryu::Buffer;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Bitcoin, CentsSigned, Sats};
use crate::{Cents, CheckedSub};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// US Dollar amount
#[derive(Debug, Default, Clone, Copy, Deref, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
#[cfg_attr(feature = "schemars", schemars(
    example = &0.0,
    example = &100.50,
    example = &30_000.0,
    example = &69_000.0,
    example = &84_342.12
))]
pub struct Dollars(f64);

impl Hash for Dollars {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Dollars {
    pub const ZERO: Self = Self(0.0);
    pub const NAN: Self = Self(f64::NAN);

    pub fn to_cents(self) -> Cents {
        Cents::from(self)
    }
}

impl From<f32> for Dollars {
    #[inline]
    fn from(value: f32) -> Self {
        Self(value as f64)
    }
}

impl From<f64> for Dollars {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<Dollars> for f32 {
    #[inline]
    fn from(value: Dollars) -> Self {
        value.0 as f32
    }
}

impl From<Dollars> for f64 {
    #[inline]
    fn from(value: Dollars) -> Self {
        value.0
    }
}

impl From<usize> for Dollars {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as f64)
    }
}

impl Add for Dollars {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Dollars {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Div<usize> for Dollars {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        if self.is_nan() || rhs == 0 {
            Dollars::NAN
        } else {
            Self::from(CentsSigned::from(self) / rhs)
        }
    }
}

impl Div<f64> for Dollars {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        if self.is_nan() || rhs == 0.0 {
            Dollars::NAN
        } else {
            Dollars::from(CentsSigned::from(Self::from(self.0 / rhs)))
        }
    }
}

impl Div<Bitcoin> for Dollars {
    type Output = Self;
    fn div(self, rhs: Bitcoin) -> Self::Output {
        let rhs = f64::from(rhs);
        if self.is_nan() || rhs == 0.0 {
            Dollars::NAN
        } else {
            Self(f64::from(self) / rhs)
        }
    }
}

impl Mul<Dollars> for Dollars {
    type Output = Self;
    fn mul(self, rhs: Dollars) -> Self::Output {
        Self::from(CentsSigned::from(self) * CentsSigned::from(rhs))
    }
}

impl Mul<f64> for Dollars {
    type Output = Dollars;
    fn mul(self, rhs: f64) -> Self::Output {
        if rhs.fract() != 0.0 {
            Self::from(self.0 * rhs)
        } else {
            self * rhs as i64
        }
    }
}

impl Mul<Bitcoin> for Dollars {
    type Output = Self;
    fn mul(self, rhs: Bitcoin) -> Self::Output {
        self * Sats::from(rhs)
    }
}

impl Mul<Sats> for Dollars {
    type Output = Self;
    fn mul(self, rhs: Sats) -> Self::Output {
        if self.is_nan() {
            self
        } else {
            let cents = i128::from(CentsSigned::from(self));
            let sats = rhs.as_u128() as i128;
            Self::from(CentsSigned::from(sats * cents / Sats::ONE_BTC_U128 as i128))
        }
    }
}

impl Mul<i64> for Dollars {
    type Output = Self;
    fn mul(self, rhs: i64) -> Self::Output {
        Self::from(CentsSigned::from(self) * rhs)
    }
}

impl Mul<usize> for Dollars {
    type Output = Self;
    fn mul(self, rhs: usize) -> Self::Output {
        if self.is_nan() {
            self
        } else {
            Self::from(CentsSigned::from(self) * rhs)
        }
    }
}

impl From<u64> for Dollars {
    #[inline]
    fn from(value: u64) -> Self {
        Self::from(CentsSigned::from(value))
    }
}

impl From<u128> for Dollars {
    #[inline]
    fn from(value: u128) -> Self {
        Self::from(CentsSigned::from(value))
    }
}

impl From<Dollars> for u128 {
    #[inline]
    fn from(value: Dollars) -> Self {
        u128::from(CentsSigned::from(value))
    }
}

impl AddAssign for Dollars {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Dollars {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl CheckedSub for Dollars {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        if self.is_nan() {
            Some(self)
        } else {
            CentsSigned::from(self)
                .checked_sub(CentsSigned::from(rhs))
                .map(Dollars::from)
        }
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for Dollars {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl CheckedSub<usize> for Dollars {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        Some(Dollars::from(
            CentsSigned::from(self)
                .checked_sub(CentsSigned::from(rhs))
                .unwrap(),
        ))
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<usize> for Dollars {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Neg for Dollars {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl PartialEq for Dollars {
    fn eq(&self, other: &Self) -> bool {
        match (self.0.is_nan(), other.0.is_nan()) {
            (true, true) => true,
            (true, false) => false,
            (false, true) => false,
            (false, false) => self.0 == other.0,
        }
    }
}

impl Eq for Dollars {}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl PartialOrd for Dollars {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for Dollars {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.0.is_nan(), other.0.is_nan()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => self.0.partial_cmp(&other.0).unwrap(),
        }
    }
}

impl Sum for Dollars {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let dollars: f64 = iter.map(|dollars| dollars.0).sum();
        Self::from(dollars)
    }
}

impl Display for Dollars {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Dollars {
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
