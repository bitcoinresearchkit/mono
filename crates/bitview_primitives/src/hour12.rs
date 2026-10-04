use std::{
    fmt::{Display, Formatter, Result},
    ops::Add,
};

use brk_types::{CheckedSub, Timestamp};
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::INDEX_EPOCH;

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex};

pub(crate) const HOUR12_INTERVAL: u32 = 43200;

/// Index of a 12-hour period.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Hour12(u32);

impl Hour12 {
    pub fn from_timestamp(ts: Timestamp) -> Self {
        Self((*ts - INDEX_EPOCH) / HOUR12_INTERVAL)
    }

    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::new(INDEX_EPOCH + self.0 * HOUR12_INTERVAL)
    }
}

impl From<Hour12> for usize {
    #[inline]
    fn from(value: Hour12) -> Self {
        value.0 as usize
    }
}

impl From<usize> for Hour12 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl Add<usize> for Hour12 {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs as u32)
    }
}

impl CheckedSub for Hour12 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for Hour12 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Hour12 {
    pub(crate) fn index_name() -> &'static str {
        "hour12"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["12h", "hour12"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for Hour12 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

impl Display for Hour12 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Hour12 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
