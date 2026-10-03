use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    ops::{Add, AddAssign, Div},
};

use brk_error::{Error, Result};
use brk_types::{CheckedSub, Timestamp};
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Date, Day1, Year1};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Year10(u8);

impl Year10 {
    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::from(Date::from(*self))
    }
}

impl From<u8> for Year10 {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<Year10> for u8 {
    #[inline]
    fn from(value: Year10) -> Self {
        value.0
    }
}

impl From<usize> for Year10 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u8)
    }
}

impl From<Year10> for usize {
    #[inline]
    fn from(value: Year10) -> Self {
        value.0 as usize
    }
}

impl Add<usize> for Year10 {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::from(self.0 + rhs as u8)
    }
}

impl Add<Year10> for Year10 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from(self.0 + rhs.0)
    }
}

impl AddAssign for Year10 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self(self.0 + rhs.0)
    }
}

impl Div<usize> for Year10 {
    type Output = Self;
    fn div(self, _: usize) -> Self::Output {
        unreachable!()
    }
}

impl From<Day1> for Year10 {
    #[inline]
    fn from(value: Day1) -> Self {
        Self::from(usize::from((Date::from(value).year() - 2000) / 10))
    }
}

impl TryFrom<Date> for Year10 {
    type Error = Error;

    #[inline]
    fn try_from(value: Date) -> Result<Self> {
        value.try_into_jiff()?;
        let years = value
            .year()
            .checked_sub(2000)
            .ok_or(Error::UnindexableDate)?;
        u8::try_from(years / 10)
            .map(Self)
            .map_err(|_| Error::UnindexableDate)
    }
}

impl CheckedSub for Year10 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for Year10 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl From<Year1> for Year10 {
    #[inline]
    fn from(value: Year1) -> Self {
        let v = usize::from(value);
        if v == 0 {
            Self(0)
        } else {
            Self((((v - 1) / 10) + 1) as u8)
        }
    }
}

impl Year10 {
    pub(crate) fn index_name() -> &'static str {
        "year10"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["10y", "decade", "year10", "decadeindex"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for Year10 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

impl Display for Year10 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Year10 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
