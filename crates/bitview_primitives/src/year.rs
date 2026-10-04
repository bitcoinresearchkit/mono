use std::{
    fmt::{Debug, Display, Formatter, Result},
    ops::{Add, AddAssign},
};

use brk_types::{CheckedSub, Timestamp};
use itoa::Buffer;
use serde::{Deserialize, Serialize};

use crate::Date;

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex};

/// Bitcoin year (2009, 2010, ..., 2025+)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash, Serialize, Deserialize,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Year(u16);

impl From<u16> for Year {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<usize> for Year {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u16)
    }
}

impl From<Year> for usize {
    #[inline]
    fn from(value: Year) -> Self {
        value.0 as usize
    }
}

impl From<Year> for u16 {
    #[inline]
    fn from(value: Year) -> Self {
        value.0
    }
}

impl Add for Year {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from(self.0 + rhs.0)
    }
}

impl AddAssign for Year {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Add<usize> for Year {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::from(self.0 + rhs as u16)
    }
}

impl From<Timestamp> for Year {
    #[inline]
    fn from(value: Timestamp) -> Self {
        Self(Date::from(value).year())
    }
}

impl From<Date> for Year {
    #[inline]
    fn from(value: Date) -> Self {
        Self(value.year())
    }
}

impl CheckedSub for Year {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for Year {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

#[cfg(feature = "storage")]
impl PrintableIndex for Year {
    fn to_string() -> &'static str {
        "year"
    }
    fn to_possible_strings() -> &'static [&'static str] {
        &["year"]
    }
}

impl Display for Year {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Year {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
