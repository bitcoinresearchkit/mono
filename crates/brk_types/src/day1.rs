use std::{
    fmt,
    ops::{Add, Rem},
};

use brk_error::{Error, Result};
use itoa::Buffer;
use jiff::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Date, Timestamp};
use crate::{CheckedSub, FromCoarserIndex, Month1, Month3, Month6, Week1, Year1, Year10};

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

impl FromCoarserIndex<Week1> for Day1 {
    fn min_from(coarser: Week1) -> usize {
        usize::from(coarser) * 7
    }

    fn max_from_(coarser: Week1) -> usize {
        usize::from(coarser) * 7 + 6
    }
}

impl FromCoarserIndex<Month1> for Day1 {
    fn min_from(coarser: Month1) -> usize {
        let d = Date::new(2009, 1, 1)
            .into_jiff()
            .checked_add(Span::new().months(u16::from(coarser)))
            .unwrap();
        Day1::try_from(Date::from(d)).unwrap().into()
    }

    fn max_from_(coarser: Month1) -> usize {
        let d = Date::new(2009, 1, 31)
            .into_jiff()
            .checked_add(Span::new().months(u16::from(coarser)))
            .unwrap();
        Day1::try_from(Date::from(d)).unwrap().into()
    }
}

impl FromCoarserIndex<Month3> for Day1 {
    fn min_from(coarser: Month3) -> usize {
        let d = Date::new(2009, 1, 1)
            .into_jiff()
            .checked_add(Span::new().months(3 * u8::from(coarser)))
            .unwrap();
        Day1::try_from(Date::from(d)).unwrap().into()
    }

    fn max_from_(coarser: Month3) -> usize {
        let d = Date::new(2009, 3, 31)
            .into_jiff()
            .checked_add(Span::new().months(3 * u8::from(coarser)))
            .unwrap();
        Day1::try_from(Date::from(d)).unwrap().into()
    }
}

impl FromCoarserIndex<Month6> for Day1 {
    fn min_from(coarser: Month6) -> usize {
        let d = Date::new(2009, 1, 1)
            .into_jiff()
            .checked_add(Span::new().months(6 * u8::from(coarser)))
            .unwrap();
        Day1::try_from(Date::from(d)).unwrap().into()
    }

    fn max_from_(coarser: Month6) -> usize {
        let d = Date::new(2009, 5, 31)
            .into_jiff()
            .checked_add(Span::new().months(1 + 6 * u8::from(coarser)))
            .unwrap();
        Day1::try_from(Date::from(d)).unwrap().into()
    }
}

impl FromCoarserIndex<Year1> for Day1 {
    fn min_from(coarser: Year1) -> usize {
        Self::try_from(Date::new(2009 + u8::from(coarser) as u16, 1, 1))
            .unwrap()
            .into()
    }

    fn max_from_(coarser: Year1) -> usize {
        Self::try_from(Date::new(2009 + u8::from(coarser) as u16, 12, 31))
            .unwrap()
            .into()
    }
}

impl FromCoarserIndex<Year10> for Day1 {
    fn min_from(coarser: Year10) -> usize {
        let coarser = u8::from(coarser);
        if coarser == 0 {
            // Decade 0 starts at 2000, before INDEX_ZERO (2009-01-01)
            0
        } else {
            Self::try_from(Date::new(2000 + 10 * coarser as u16, 1, 1))
                .unwrap()
                .into()
        }
    }

    fn max_from_(coarser: Year10) -> usize {
        let coarser = u8::from(coarser);
        Self::try_from(Date::new(2009 + 10 * coarser as u16, 12, 31))
            .unwrap()
            .into()
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

#[cfg(test)]
#[path = "../tests/unit/day1.rs"]
mod tests;
