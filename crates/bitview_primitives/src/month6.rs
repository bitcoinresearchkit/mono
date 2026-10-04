use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    ops::{Add, AddAssign, Div},
};

use brk_error::{Error, Result};
use brk_types::{CheckedSub, Timestamp};
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Date, Month1};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex};

/// Index of a half-year (6 months).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Month6(u8);

impl Month6 {
    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::from(Date::from(*self))
    }
}

impl From<u8> for Month6 {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<usize> for Month6 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u8)
    }
}

impl From<Month6> for u8 {
    #[inline]
    fn from(value: Month6) -> Self {
        value.0
    }
}

impl From<Month6> for usize {
    #[inline]
    fn from(value: Month6) -> Self {
        value.0 as usize
    }
}

impl Add<usize> for Month6 {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::from(self.0 + rhs as u8)
    }
}

impl Add<Month6> for Month6 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from(self.0 + rhs.0)
    }
}

impl AddAssign for Month6 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self(self.0 + rhs.0)
    }
}

impl Div<usize> for Month6 {
    type Output = Self;
    fn div(self, _: usize) -> Self::Output {
        unreachable!()
    }
}

impl TryFrom<Date> for Month6 {
    type Error = Error;

    #[inline]
    fn try_from(value: Date) -> Result<Self> {
        let months = u16::from(Month1::try_from(value)?);
        u8::try_from(months / 6)
            .map(Self)
            .map_err(|_| Error::UnindexableDate)
    }
}

impl From<Month1> for Month6 {
    #[inline]
    fn from(value: Month1) -> Self {
        Self((usize::from(value) / 6) as u8)
    }
}

impl CheckedSub for Month6 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for Month6 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Month6 {
    pub(crate) fn index_name() -> &'static str {
        "month6"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["semester", "s", "month6", "semesterindex", "6m", "6mo"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for Month6 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

impl Display for Month6 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Month6 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
