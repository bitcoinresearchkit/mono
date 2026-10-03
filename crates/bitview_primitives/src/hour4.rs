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

pub const HOUR4_INTERVAL: u32 = 14400;

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Hour4(u32);

impl Hour4 {
    pub fn from_timestamp(ts: Timestamp) -> Self {
        Self((*ts - INDEX_EPOCH) / HOUR4_INTERVAL)
    }

    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::new(INDEX_EPOCH + self.0 * HOUR4_INTERVAL)
    }
}

impl From<Hour4> for usize {
    #[inline]
    fn from(value: Hour4) -> Self {
        value.0 as usize
    }
}

impl From<usize> for Hour4 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl Add<usize> for Hour4 {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs as u32)
    }
}

impl CheckedSub for Hour4 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for Hour4 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Hour4 {
    pub(crate) fn index_name() -> &'static str {
        "hour4"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["4h", "hour4"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for Hour4 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

impl Display for Hour4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Hour4 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
