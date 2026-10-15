use std::{
    hash::{Hash, Hasher},
    ops::{Div, Mul},
};

use derive_more::{Add, AddAssign, Deref, From, Into, Neg, Sub, SubAssign, Sum};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Bitcoin, CentsSigned, Sats};
use crate::{Cents, CheckedSub};

#[cfg(feature = "storage")]
use vecdb::Pco;

/// US Dollar amount
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Deref,
    Serialize,
    Deserialize,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Neg,
    Sum,
    From,
    Into,
)]
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

/// Agrees with `Eq`: both zeros hash alike, as do all NaNs.
impl Hash for Dollars {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let bits = if self.0.is_nan() {
            Self::NAN.0.to_bits()
        } else if self.0 == 0.0 {
            0
        } else {
            self.0.to_bits()
        };
        bits.hash(state);
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

impl From<Dollars> for f32 {
    #[inline]
    fn from(value: Dollars) -> Self {
        value.0 as f32
    }
}

impl From<usize> for Dollars {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as f64)
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

impl Mul<f64> for Dollars {
    type Output = Dollars;
    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs)
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
        Self(self.0 * rhs as f64)
    }
}

impl Mul<usize> for Dollars {
    type Output = Self;
    fn mul(self, rhs: usize) -> Self::Output {
        Self(self.0 * rhs as f64)
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

/// An undefined `self` stays undefined; subtracting an undefined `rhs` subtracts zero (running
/// sums stay alive).
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

crate::impl_float_ord_format!(Dollars);
