use std::{
    fmt::{Display, Formatter, Result},
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul},
};

use brk_types::{Bitcoin, Cents, CentsSigned, Dollars, Sats};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::Serialize;

use crate::StoredF64;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Closing price value for a time period
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Deref,
    DerefMut,
    Serialize,
    JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
#[schemars(transparent)]
#[repr(transparent)]
pub struct Close<T>(T);

impl<T> Close<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> From<usize> for Close<T>
where
    T: From<usize>,
{
    #[inline]
    fn from(value: usize) -> Self {
        Self(T::from(value))
    }
}

impl<T> From<f32> for Close<T>
where
    T: From<f32>,
{
    #[inline]
    fn from(value: f32) -> Self {
        Self(T::from(value))
    }
}

impl<T> From<f64> for Close<T>
where
    T: From<f64>,
{
    #[inline]
    fn from(value: f64) -> Self {
        Self(T::from(value))
    }
}

impl<T> From<Close<T>> for f32
where
    f32: From<T>,
{
    #[inline]
    fn from(value: Close<T>) -> Self {
        Self::from(value.0)
    }
}

impl<T> From<Close<T>> for f64
where
    f64: From<T>,
{
    #[inline]
    fn from(value: Close<T>) -> Self {
        Self::from(value.0)
    }
}

impl<T> From<Close<T>> for StoredF64
where
    StoredF64: From<T>,
{
    #[inline]
    fn from(value: Close<T>) -> Self {
        Self::from(value.0)
    }
}

impl From<Close<Cents>> for Close<Dollars> {
    #[inline]
    fn from(value: Close<Cents>) -> Self {
        Self(Dollars::from(*value))
    }
}

impl<T> Add for Close<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<T> AddAssign for Close<T>
where
    T: Add<Output = T> + Clone,
{
    fn add_assign(&mut self, rhs: Self) {
        **self = self.0.clone() + rhs.0
    }
}

impl<T> Div<usize> for Close<T>
where
    T: Div<usize, Output = T>,
{
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl Sum for Close<Dollars> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(Dollars::from(iter.map(|v| f64::from(v.0)).sum::<f64>()))
    }
}

impl<T> Display for Close<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "storage")]
impl<T> Formattable for Close<T>
where
    T: Display,
{
    fn write_to(&self, buf: &mut Vec<u8>) {
        use std::fmt::Write;
        let mut s = String::new();
        write!(s, "{}", self).unwrap();
        buf.extend_from_slice(s.as_bytes());
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}

impl From<Close<Dollars>> for Dollars {
    #[inline]
    fn from(value: Close<Dollars>) -> Self {
        *value
    }
}

impl Div<Close<Dollars>> for Dollars {
    type Output = StoredF64;
    fn div(self, rhs: Close<Dollars>) -> Self::Output {
        if self.is_nan() || *rhs == Dollars::ZERO {
            StoredF64::NAN
        } else {
            StoredF64::from(f64::from(self) / f64::from(*rhs))
        }
    }
}

impl Div<Dollars> for Close<Dollars> {
    type Output = StoredF64;
    fn div(self, rhs: Dollars) -> Self::Output {
        if self.is_nan() || rhs == Dollars::ZERO {
            StoredF64::NAN
        } else {
            StoredF64::from(f64::from(*self) / f64::from(rhs))
        }
    }
}

impl Mul<Close<Dollars>> for Dollars {
    type Output = Self;
    fn mul(self, rhs: Close<Dollars>) -> Self::Output {
        Self::from(CentsSigned::from(self) * CentsSigned::from(*rhs))
    }
}

impl Mul<Dollars> for Close<Dollars> {
    type Output = Dollars;
    fn mul(self, rhs: Dollars) -> Self::Output {
        Dollars::from(CentsSigned::from(*self) * CentsSigned::from(rhs))
    }
}

impl Mul<usize> for Close<Dollars> {
    type Output = Dollars;
    fn mul(self, rhs: usize) -> Self::Output {
        Dollars::from(CentsSigned::from(*self) * rhs)
    }
}

impl Mul<StoredF64> for Close<Dollars> {
    type Output = Dollars;
    fn mul(self, rhs: StoredF64) -> Self::Output {
        *self * rhs
    }
}

impl Mul<Bitcoin> for Close<Dollars> {
    type Output = Dollars;
    fn mul(self, rhs: Bitcoin) -> Self::Output {
        *self * Sats::from(rhs)
    }
}

impl From<Close<Dollars>> for u128 {
    #[inline]
    fn from(value: Close<Dollars>) -> Self {
        u128::from(*value)
    }
}
