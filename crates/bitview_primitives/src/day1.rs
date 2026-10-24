use std::ops::Rem;

use brk_error::{Error, Result};
use brk_types::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Date;

#[cfg(feature = "storage")]
use vecdb::{Pco, PrintableIndex};

/// Index of a day.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Day1(u16);

brk_types::impl_integer_index!(Day1, u16);

impl Day1 {
    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::from(Date::from(*self))
    }

    pub(crate) fn index_name() -> &'static str {
        "day1"
    }
}

impl From<Day1> for u64 {
    #[inline]
    fn from(value: Day1) -> Self {
        value.0 as u64
    }
}

impl From<Day1> for i64 {
    #[inline]
    fn from(value: Day1) -> Self {
        value.0 as i64
    }
}

impl TryFrom<Date> for Day1 {
    type Error = Error;
    fn try_from(value: Date) -> Result<Self> {
        let days = Date::JIFF_INDEX_ZERO
            .until(value.try_into_jiff()?)?
            .get_days();
        u16::try_from(days)
            .map(Self)
            .map_err(|_| Error::UnindexableDate)
    }
}

impl Rem<usize> for Day1 {
    type Output = Self;
    fn rem(self, rhs: usize) -> Self::Output {
        Self(self.0 % rhs as u16)
    }
}

#[cfg(feature = "storage")]
impl PrintableIndex for Day1 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
}
