use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use derive_more::Deref;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Dollars, Sats};
use crate::{CheckedSub, unlikely};

#[cfg(feature = "storage")]
use vecdb::Pco;

/// An amount in US cents (100 cents = 1 USD).
///
/// Unsigned; `u64::MAX` is reserved as a NaN sentinel.
#[derive(
    Debug, Default, Deref, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Cents(u64);

impl Cents {
    pub const ZERO: Self = Self(0);
    pub const MAX_FINITE: Self = Self(u64::MAX - 1);
    pub const NAN: Self = Self(u64::MAX);
    #[inline]
    pub const fn new(value: u64) -> Self {
        debug_assert!(value != u64::MAX, "u64::MAX is reserved as Cents::NAN");
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
        debug_assert!(
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

    /// The whole sats one dollar buys at this price (cents per BTC), rounded down; zero at a zero
    /// price. A NaN price has no whole-sats value.
    #[inline]
    pub fn sats_per_dollar(self) -> Sats {
        debug_assert!(!self.is_nan(), "a NaN price has no sats per dollar");
        Sats::new(
            (100 * Sats::ONE_BTC_U64)
                .checked_div(self.inner())
                .unwrap_or_default(),
        )
    }

    pub fn to_dollars(self) -> Dollars {
        if unlikely(self.is_nan()) {
            Dollars::NAN
        } else {
            Dollars::from(self.0 as f64 / 100.0)
        }
    }

    /// Rounds half up to `digits` significant digits of the amount in cents; amounts under
    /// 10^`digits` cents stay exact. With 5 digits, 12345 (= $123.45) stays and 1234567
    /// (= $12,345.67) becomes 1234600 (= $12,346.00).
    #[inline]
    pub fn round_to_significant(self, digits: u32) -> Self {
        if unlikely(self.is_nan()) {
            return Self::NAN;
        }
        let cents = self.0 as u128;
        match cents.checked_ilog10() {
            Some(ilog10) if ilog10 >= digits => {
                let pow = 10u128.pow(ilog10 - digits + 1);
                Self::from_finite_u128((cents + pow / 2) / pow * pow)
            }
            _ => self,
        }
    }
}

impl From<Dollars> for Cents {
    #[inline]
    fn from(value: Dollars) -> Self {
        Self::from(f64::from(value) * 100.0)
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
        Self::new(value as u64)
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

/// Rounds to the nearest cent.
impl From<f64> for Cents {
    #[inline]
    fn from(value: f64) -> Self {
        let value = value.round();
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

impl CheckedSub for Cents {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        Cents::checked_sub(self, rhs)
    }
}

crate::impl_int_format!(Cents, nullable);
