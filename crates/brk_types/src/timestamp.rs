use bitcoin::locktime::absolute::Time;
use derive_more::{Add, AddAssign, Deref, From};
use jiff::Timestamp as JiffTimestamp;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::CheckedSub;

#[cfg(feature = "storage")]
use vecdb::Pco;

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
    Add,
    AddAssign,
    From,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
#[cfg_attr(feature = "schemars", schemars(
    example = &1231006505,
    example = &1672531200,
    example = &1713571200,
    example = &1743631892,
    example = &1759000868
))]
pub struct Timestamp(u32);

pub const ONE_HOUR_IN_SEC: u32 = 60 * 60;
pub const ONE_DAY_IN_SEC: u32 = 24 * 60 * 60;
pub const ONE_DAY_IN_SEC_F64: f64 = ONE_DAY_IN_SEC as f64;

impl Timestamp {
    pub const ZERO: Self = Self(0);

    pub fn new(timestamp: u32) -> Self {
        Self(timestamp)
    }

    #[inline]
    pub fn days_since(&self, older: Self) -> f64 {
        (self.0 - older.0) as f64 / ONE_DAY_IN_SEC_F64
    }

    #[inline]
    pub fn hours_since(&self, older: Self) -> usize {
        ((self.0 - older.0) / ONE_HOUR_IN_SEC) as usize
    }

    pub fn now() -> Self {
        Self::from(JiffTimestamp::now())
    }

    pub fn to_iso8601(self) -> String {
        JiffTimestamp::from(self).to_string()
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

impl CheckedSub<Timestamp> for Timestamp {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
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

crate::impl_int_format!(Timestamp);
