use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Div},
};

use brk_types::{Cents, Dollars};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::Serialize;

use super::Close;
use crate::StoredF64;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Lowest price value for a time period
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
pub struct Low<T>(T);

impl<T> Low<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> From<usize> for Low<T>
where
    T: From<usize>,
{
    #[inline]
    fn from(value: usize) -> Self {
        Self(T::from(value))
    }
}

impl<T> From<f64> for Low<T>
where
    T: From<f64>,
{
    #[inline]
    fn from(value: f64) -> Self {
        Self(T::from(value))
    }
}

impl<T> From<Low<T>> for f64
where
    f64: From<T>,
{
    #[inline]
    fn from(value: Low<T>) -> Self {
        Self::from(value.0)
    }
}

impl<T> From<Low<T>> for StoredF64
where
    StoredF64: From<T>,
{
    #[inline]
    fn from(value: Low<T>) -> Self {
        Self::from(value.0)
    }
}

impl<T> From<Close<T>> for Low<T>
where
    T: Copy,
{
    #[inline]
    fn from(value: Close<T>) -> Self {
        Self(*value)
    }
}

impl From<Low<Cents>> for Low<Dollars> {
    #[inline]
    fn from(value: Low<Cents>) -> Self {
        Self(Dollars::from(*value))
    }
}

impl<T> Add for Low<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<T> AddAssign for Low<T>
where
    T: Add<Output = T> + Clone,
{
    fn add_assign(&mut self, rhs: Self) {
        **self = self.0.clone() + rhs.0
    }
}

impl<T> Div<usize> for Low<T>
where
    T: Div<usize, Output = T>,
{
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl<T> Display for Low<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "storage")]
impl<T> Formattable for Low<T>
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

impl From<Low<Dollars>> for Dollars {
    #[inline]
    fn from(value: Low<Dollars>) -> Self {
        *value
    }
}
