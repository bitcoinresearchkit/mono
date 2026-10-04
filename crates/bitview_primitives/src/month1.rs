use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    ops::{Add, AddAssign, Div},
};

use brk_error::{Error, Result};
use brk_types::{CheckedSub, Timestamp};
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Date, Day1};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex};

/// Index of a month.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Month1(u16);

impl Month1 {
    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::from(Date::from(*self))
    }
}

impl From<u16> for Month1 {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<Month1> for u16 {
    #[inline]
    fn from(value: Month1) -> Self {
        value.0
    }
}

impl From<usize> for Month1 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u16)
    }
}

impl From<Month1> for u64 {
    #[inline]
    fn from(value: Month1) -> Self {
        value.0 as u64
    }
}

impl From<Month1> for usize {
    #[inline]
    fn from(value: Month1) -> Self {
        value.0 as usize
    }
}

impl Add<usize> for Month1 {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::from(self.0 + rhs as u16)
    }
}

impl Add<Month1> for Month1 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from(self.0 + rhs.0)
    }
}

impl AddAssign for Month1 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self(self.0 + rhs.0)
    }
}

impl Div<usize> for Month1 {
    type Output = Self;
    fn div(self, _: usize) -> Self::Output {
        unreachable!()
    }
}

impl From<Day1> for Month1 {
    #[inline]
    fn from(value: Day1) -> Self {
        let date = Date::from(value);
        Self::from(usize::from(date.year() - 2009) * 12 + usize::from(date.month()) - 1)
    }
}

impl TryFrom<Date> for Month1 {
    type Error = Error;

    #[inline]
    fn try_from(value: Date) -> Result<Self> {
        value.try_into_jiff()?;
        let years = value
            .year()
            .checked_sub(2009)
            .ok_or(Error::UnindexableDate)?;
        let months = u32::from(years) * 12 + u32::from(value.month()) - 1;
        u16::try_from(months)
            .map(Self)
            .map_err(|_| Error::UnindexableDate)
    }
}

impl CheckedSub for Month1 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for Month1 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Month1 {
    pub(crate) fn index_name() -> &'static str {
        "month1"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["month", "m", "monthly", "month1", "monthindex", "1m", "1mo"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for Month1 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

impl Display for Month1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Month1 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
