use std::{
    fmt::{Display, Formatter, Result},
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use derive_more::Deref;
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{CentsSats, Dollars, Sats, StoredF64};
use crate::{CheckedSub, unlikely};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Unsigned cents (u64) - for values that should never be negative.
/// Used for invested capital, realized cap, etc.
/// `u64::MAX` is reserved as a NaN sentinel.
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
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Cents(u64);

impl Cents {
    pub const ZERO: Self = Self(0);
    pub const MAX_FINITE: Self = Self(u64::MAX - 1);
    pub const NAN: Self = Self(u64::MAX);
    #[inline]
    pub const fn new(value: u64) -> Self {
        assert!(value != u64::MAX, "u64::MAX is reserved as Cents::NAN");
        Self(value)
    }

    #[inline]
    pub const fn is_nan(self) -> bool {
        self.0 == u64::MAX
    }

    #[inline]
    pub const fn finite_inner(self) -> Option<u64> {
        if self.is_nan() { None } else { Some(self.0) }
    }

    #[inline]
    pub const fn inner(self) -> u64 {
        match self.finite_inner() {
            Some(value) => value,
            None => panic!("Cents::NAN has no finite integer representation"),
        }
    }

    #[inline]
    pub const fn as_u128(self) -> u128 {
        self.inner() as u128
    }

    #[inline]
    fn checked_finite(value: Option<u64>) -> Option<Self> {
        match value {
            Some(u64::MAX) | None => None,
            Some(value) => Some(Self(value)),
        }
    }

    #[inline]
    fn from_finite_u128(value: u128) -> Self {
        assert!(
            value < u64::MAX as u128,
            "u128 overflow or Cents::NAN sentinel collision"
        );
        Self(value as u64)
    }

    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        if unlikely(self.is_nan() || rhs.is_nan()) {
            Some(Self::NAN)
        } else {
            self.0.checked_sub(rhs.0).map(Self)
        }
    }

    #[inline]
    pub fn saturating_sub(self, rhs: Self) -> Self {
        if unlikely(self.is_nan() || rhs.is_nan()) {
            Self::NAN
        } else {
            Self(self.0.saturating_sub(rhs.0))
        }
    }

    #[inline]
    fn checked_add(self, rhs: Self) -> Option<Self> {
        if unlikely(self.is_nan() || rhs.is_nan()) {
            Some(Self::NAN)
        } else {
            Self::checked_finite(self.0.checked_add(rhs.0))
        }
    }

    pub fn to_dollars(self) -> Dollars {
        if unlikely(self.is_nan()) {
            Dollars::NAN
        } else {
            Dollars::from(self.0 as f64 / 100.0)
        }
    }

    /// Round to nearest dollar, then apply N significant digits.
    /// E.g., 12345 (= $123.45) → 12300 (= $123.00) with 5 digits
    /// E.g., 1234567 (= $12345.67) → 1234600 (= $12346.00) with 5 digits
    #[inline]
    pub fn round_to_dollar(self, digits: i32) -> Self {
        if unlikely(self.is_nan()) {
            return Self::NAN;
        }
        // Round to nearest dollar (nearest 100 cents)
        let dollars = (self.0 as u128 + 50) / 100;
        // Apply significant digit rounding to dollars, then convert back to cents
        let ilog10 = dollars.checked_ilog10().unwrap_or(0) as i32;
        let rounded_dollars = if ilog10 >= digits {
            let log_diff = ilog10 - digits + 1;
            let pow = 10u128.pow(log_diff as u32);
            ((dollars + pow / 2) / pow) * pow
        } else {
            dollars
        };
        Self::from_finite_u128(rounded_dollars * 100)
    }
}

impl From<Dollars> for Cents {
    #[inline]
    fn from(value: Dollars) -> Self {
        let f = f64::from(value);
        if unlikely(!f.is_finite()) {
            Self::NAN
        } else if f < 0.0 {
            Self::ZERO
        } else {
            let cents = (f * 100.0).round();
            if cents >= u64::MAX as f64 {
                Self::MAX_FINITE
            } else {
                Self(cents as u64)
            }
        }
    }
}

impl From<Cents> for Dollars {
    #[inline]
    fn from(value: Cents) -> Self {
        value.to_dollars()
    }
}

impl From<u64> for Cents {
    #[inline]
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl From<usize> for Cents {
    #[inline]
    fn from(value: usize) -> Self {
        Self::new(u64::try_from(value).expect("usize overflow to Cents"))
    }
}

impl From<Cents> for u64 {
    #[inline]
    fn from(value: Cents) -> Self {
        value.inner()
    }
}

impl From<u128> for Cents {
    #[inline]
    fn from(value: u128) -> Self {
        Self::from_finite_u128(value)
    }
}

impl From<Cents> for u128 {
    #[inline]
    fn from(value: Cents) -> Self {
        value.as_u128()
    }
}

impl From<Cents> for f64 {
    #[inline]
    fn from(value: Cents) -> Self {
        if unlikely(value.is_nan()) {
            f64::NAN
        } else {
            value.0 as f64
        }
    }
}

impl From<f64> for Cents {
    #[inline]
    fn from(value: f64) -> Self {
        if unlikely(!value.is_finite()) {
            Self::NAN
        } else if value < 0.0 {
            Self::ZERO
        } else if value >= u64::MAX as f64 {
            Self::MAX_FINITE
        } else {
            Self(value as u64)
        }
    }
}

impl Add for Cents {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(rhs).expect("Cents overflow")
    }
}

impl AddAssign for Cents {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sum for Cents {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |sum, value| sum + value)
    }
}

impl Sub for Cents {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs).expect("Cents underflow")
    }
}

impl SubAssign for Cents {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for Cents {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        if unlikely(self.is_nan() || rhs.is_nan()) {
            Self::NAN
        } else {
            Self::checked_finite(self.0.checked_mul(rhs.0)).expect("Cents overflow")
        }
    }
}

impl Mul<u64> for Cents {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: u64) -> Self::Output {
        if unlikely(self.is_nan()) {
            Self::NAN
        } else {
            Self::checked_finite(self.0.checked_mul(rhs)).expect("Cents overflow")
        }
    }
}

impl Mul<usize> for Cents {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: usize) -> Self::Output {
        if unlikely(self.is_nan()) {
            Self::NAN
        } else {
            let rhs = u64::try_from(rhs).expect("usize overflow to Cents multiplier");
            Self::checked_finite(self.0.checked_mul(rhs)).expect("Cents overflow")
        }
    }
}

impl Mul<StoredF64> for Cents {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: StoredF64) -> Self::Output {
        Self::from(f64::from(self) * f64::from(rhs))
    }
}

impl Mul<Sats> for Cents {
    type Output = CentsSats;
    #[inline]
    fn mul(self, sats: Sats) -> CentsSats {
        CentsSats::new(self.as_u128() * sats.as_u128())
    }
}

impl Div<Cents> for Cents {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        if unlikely(self.is_nan() || rhs.is_nan()) {
            Self::NAN
        } else {
            Self(self.0 / rhs.0)
        }
    }
}

impl Div<u64> for Cents {
    type Output = Self;
    #[inline]
    fn div(self, rhs: u64) -> Self::Output {
        if unlikely(self.is_nan()) {
            Self::NAN
        } else {
            Self(self.0 / rhs)
        }
    }
}

impl Div<usize> for Cents {
    type Output = Self;
    #[inline]
    fn div(self, rhs: usize) -> Self::Output {
        if unlikely(self.is_nan()) {
            Self::NAN
        } else {
            Self(self.0 / rhs as u64)
        }
    }
}

impl CheckedSub for Cents {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        Cents::checked_sub(self, rhs)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for Cents {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Display for Cents {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Cents {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }

    #[inline(always)]
    fn fmt_json(&self, buf: &mut Vec<u8>) {
        if unlikely(self.is_nan()) {
            buf.extend_from_slice(b"null");
        } else {
            self.write_to(buf);
        }
    }
}
