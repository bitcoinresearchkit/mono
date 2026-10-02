use std::{fmt, ops::Add};

use brk_error::{Error, Result};
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Date, INDEX_EPOCH, Timestamp};
use crate::CheckedSub;

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex};

const DAY3_INTERVAL: u32 = 259200;

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Day3(u16);

impl Day3 {
    pub fn from_timestamp(ts: Timestamp) -> Self {
        Self(((*ts).saturating_sub(INDEX_EPOCH - 86400) / DAY3_INTERVAL) as u16)
    }

    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::new(INDEX_EPOCH - 86400 + self.0 as u32 * DAY3_INTERVAL)
    }
}

impl TryFrom<Date> for Day3 {
    type Error = Error;

    fn try_from(date: Date) -> Result<Self> {
        // Bucket zero starts one day before the daily index epoch.
        let days = Date::INDEX_ZERO_.until(date.try_into_jiff()?)?.get_days() + 1;
        let days = u32::try_from(days).map_err(|_| Error::UnindexableDate)?;
        u16::try_from(days / 3)
            .map(Self)
            .map_err(|_| Error::UnindexableDate)
    }
}

impl From<Day3> for usize {
    #[inline]
    fn from(value: Day3) -> Self {
        value.0 as usize
    }
}

impl From<usize> for Day3 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u16)
    }
}

impl Add<usize> for Day3 {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs as u16)
    }
}

impl CheckedSub for Day3 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for Day3 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Day3 {
    pub(crate) fn index_name() -> &'static str {
        "day3"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["3d", "day3"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for Day3 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

impl fmt::Display for Day3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Day3 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
