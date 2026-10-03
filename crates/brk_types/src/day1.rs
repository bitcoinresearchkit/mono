use std::{
    fmt,
    ops::{Add, Rem},
};

use brk_error::{Error, Result};
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Date, Timestamp};
use crate::CheckedSub;

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Day1(u16);

impl Day1 {
    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::from(Date::from(*self))
    }
}

impl From<Day1> for usize {
    #[inline]
    fn from(value: Day1) -> Self {
        value.0 as usize
    }
}

impl From<Day1> for u64 {
    #[inline]
    fn from(value: Day1) -> Self {
        value.0 as u64
    }
}

impl From<usize> for Day1 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u16)
    }
}

impl From<Day1> for i64 {
    #[inline]
    fn from(value: Day1) -> Self {
        value.0 as i64
    }
}

impl Add<usize> for Day1 {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs as u16)
    }
}

impl TryFrom<Date> for Day1 {
    type Error = Error;
    fn try_from(value: Date) -> Result<Self> {
        let days = Date::INDEX_ZERO_.until(value.try_into_jiff()?)?.get_days();
        u16::try_from(days)
            .map(Self)
            .map_err(|_| Error::UnindexableDate)
    }
}

impl CheckedSub for Day1 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for Day1 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Rem<usize> for Day1 {
    type Output = Self;
    fn rem(self, rhs: usize) -> Self::Output {
        Self(self.0 % rhs as u16)
    }
}

impl Day1 {
    pub(crate) fn index_name() -> &'static str {
        "day1"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["1d", "d", "day", "date", "daily", "day1", "dateindex"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for Day1 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

impl fmt::Display for Day1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Day1 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
