use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign},
};

use brk_types::{Cents, Dollars};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::Serialize;

#[cfg(feature = "storage")]
use vecdb::Pco;

mod cents;
mod dollars;

pub use cents::*;
pub use dollars::*;

/// One candle role (open, high, low, close) wrapping a price, transparent in layout and schema.
macro_rules! candle_roles {
    ($($(#[$meta:meta])* $name:ident,)*) => {$(
        $(#[$meta])*
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
        pub struct $name<T>(T);

        impl<T> $name<T> {
            pub fn new(value: T) -> Self {
                Self(value)
            }
        }

        impl<T> From<usize> for $name<T>
        where
            T: From<usize>,
        {
            #[inline]
            fn from(value: usize) -> Self {
                Self(T::from(value))
            }
        }

        impl<T> From<f64> for $name<T>
        where
            T: From<f64>,
        {
            #[inline]
            fn from(value: f64) -> Self {
                Self(T::from(value))
            }
        }

        impl<T> From<$name<T>> for f64
        where
            f64: From<T>,
        {
            #[inline]
            fn from(value: $name<T>) -> Self {
                Self::from(value.0)
            }
        }

        impl From<$name<Cents>> for $name<Dollars> {
            #[inline]
            fn from(value: $name<Cents>) -> Self {
                Self(Dollars::from(value.0))
            }
        }

        impl From<$name<Dollars>> for Dollars {
            #[inline]
            fn from(value: $name<Dollars>) -> Self {
                value.0
            }
        }

        impl<T> Add for $name<T>
        where
            T: Add<Output = T>,
        {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<T> AddAssign for $name<T>
        where
            T: Add<Output = T> + Clone,
        {
            fn add_assign(&mut self, rhs: Self) {
                self.0 = self.0.clone() + rhs.0
            }
        }

        impl<T> Display for $name<T>
        where
            T: Display,
        {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                self.0.fmt(f)
            }
        }
    )*};
}

candle_roles! {
    /// Opening price value for a time period
    Open,
    /// Highest price value for a time period
    High,
    /// Lowest price value for a time period
    Low,
    /// Closing price value for a time period
    Close,
}

/// A single price is a flat candle: every role takes the close.
macro_rules! from_close {
    ($($name:ident),*) => {$(
        impl<T> From<Close<T>> for $name<T> {
            #[inline]
            fn from(value: Close<T>) -> Self {
                Self(value.0)
            }
        }
    )*};
}

from_close!(Open, High, Low);
