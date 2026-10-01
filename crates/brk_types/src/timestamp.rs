use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Div},
};

use bitcoin::locktime::absolute::Time;
use derive_more::Deref;
use itoa::Buffer;
use jiff::{
    Timestamp as JiffTimestamp,
    civil::{Date as CivilDate, DateTime, date},
    tz::TimeZone,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Date;
use crate::CheckedSub;

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// UNIX timestamp in seconds
#[derive(
    Debug,
    Default,
    Deref,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
#[schemars(
    example = &1231006505,
    example = &1672531200,
    example = &1713571200,
    example = &1743631892,
    example = &1759000868
)]
pub struct Timestamp(u32);

pub const ONE_HOUR_IN_SEC: u32 = 60 * 60;
pub const ONE_DAY_IN_SEC: u32 = 24 * 60 * 60;
pub const ONE_DAY_IN_SEC_F64: f64 = ONE_DAY_IN_SEC as f64;

/// 2009-01-01 00:00:00 UTC — epoch for fixed-interval time indexes.
pub const INDEX_EPOCH: u32 = 1230768000;

impl Timestamp {
    pub const ZERO: Self = Self(0);

    pub fn new(timestamp: u32) -> Self {
        Self(timestamp)
    }

    pub fn floor_seconds(self) -> Self {
        let zoned = JiffTimestamp::from(self).to_zoned(TimeZone::UTC);
        let date_time = DateTime::from(zoned);
        let trunc_date_time = date(date_time.year(), date_time.month(), date_time.day()).at(
            date_time.hour(),
            date_time.minute(),
            0,
            0,
        );
        Self::from(trunc_date_time.to_zoned(TimeZone::UTC).unwrap().timestamp())
    }

    #[inline]
    pub fn difference_in_days_between_float(&self, older: Self) -> f64 {
        (self.0 - older.0) as f64 / ONE_DAY_IN_SEC_F64
    }

    #[inline]
    pub(crate) fn difference_in_hours_between(&self, older: Self) -> usize {
        ((self.0 - older.0) / ONE_HOUR_IN_SEC) as usize
    }

    pub fn now() -> Self {
        Self::from(JiffTimestamp::now())
    }

    /// Returns an ISO 8601 formatted string
    pub fn to_iso8601(self) -> String {
        JiffTimestamp::from(self).to_string()
    }
}

impl From<u32> for Timestamp {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<i64> for Timestamp {
    #[inline]
    fn from(value: i64) -> Self {
        let value = value.max(0);
        debug_assert!(value <= u32::MAX as i64);
        Self(value as u32)
    }
}

impl From<JiffTimestamp> for Timestamp {
    #[inline]
    fn from(value: JiffTimestamp) -> Self {
        Self(value.as_second() as u32)
    }
}

impl From<Timestamp> for JiffTimestamp {
    #[inline]
    fn from(value: Timestamp) -> Self {
        JiffTimestamp::from_second(*value as i64).unwrap()
    }
}

impl From<Time> for Timestamp {
    #[inline]
    fn from(value: Time) -> Self {
        Self(value.to_consensus_u32())
    }
}

impl From<usize> for Timestamp {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl From<Timestamp> for usize {
    #[inline]
    fn from(value: Timestamp) -> Self {
        value.0 as usize
    }
}

impl From<Timestamp> for u64 {
    #[inline]
    fn from(value: Timestamp) -> Self {
        u64::from(value.0)
    }
}

impl From<Date> for Timestamp {
    #[inline]
    fn from(value: Date) -> Self {
        Self::from(
            CivilDate::from(value)
                .to_zoned(TimeZone::UTC)
                .unwrap()
                .timestamp(),
        )
    }
}

impl CheckedSub<Timestamp> for Timestamp {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<Timestamp> for Timestamp {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Div<usize> for Timestamp {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs as u32)
    }
}

impl Add for Timestamp {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Timestamp {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl From<f64> for Timestamp {
    #[inline]
    fn from(value: f64) -> Self {
        let value = value.max(0.0);
        debug_assert!(value <= u32::MAX as f64);
        Self(value as u32)
    }
}

impl From<Timestamp> for f64 {
    #[inline]
    fn from(value: Timestamp) -> Self {
        value.0 as f64
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Timestamp {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
