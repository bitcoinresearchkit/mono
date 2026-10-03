use std::{
    fmt::{self, Debug},
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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Week1(u16);

impl Week1 {
    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::from(Date::from(*self))
    }
}

impl From<u16> for Week1 {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<Week1> for u16 {
    #[inline]
    fn from(value: Week1) -> Self {
        value.0
    }
}

impl From<usize> for Week1 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u16)
    }
}

impl From<Week1> for usize {
    #[inline]
    fn from(value: Week1) -> Self {
        value.0 as usize
    }
}

impl Add<Week1> for Week1 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from(self.0 + rhs.0)
    }
}

impl AddAssign for Week1 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self(self.0 + rhs.0)
    }
}

impl Div<usize> for Week1 {
    type Output = Self;
    fn div(self, _: usize) -> Self::Output {
        unreachable!()
    }
}

impl Add<usize> for Week1 {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::from(self.0 + rhs as u16)
    }
}

impl From<Day1> for Week1 {
    #[inline]
    fn from(value: Day1) -> Self {
        Self(((usize::from(value) + 3) / 7) as u16)
    }
}

impl TryFrom<Date> for Week1 {
    type Error = Error;

    #[inline]
    fn try_from(value: Date) -> Result<Self> {
        // ISO week 2009-W01 starts three days before the daily epoch.
        let days = value
            .try_into_jiff()?
            .duration_since(Date::INDEX_ZERO_)
            .as_secs()
            / 86_400
            + 3;
        let days = u32::try_from(days).map_err(|_| Error::UnindexableDate)?;
        u16::try_from(days / 7)
            .map(Self)
            .map_err(|_| Error::UnindexableDate)
    }
}

impl CheckedSub for Week1 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for Week1 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Week1 {
    pub(crate) fn index_name() -> &'static str {
        "week1"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["1w", "w", "week", "weekly", "week1", "weekindex"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for Week1 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

impl fmt::Display for Week1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Week1 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
