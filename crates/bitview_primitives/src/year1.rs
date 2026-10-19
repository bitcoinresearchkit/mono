use brk_error::{Error, Result};
use brk_types::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Date, Day1, Month1};

#[cfg(feature = "storage")]
use vecdb::{Pco, PrintableIndex};

/// Index of a year.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Year1(u8);

brk_types::impl_integer_index!(Year1, u8);

impl Year1 {
    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::from(Date::from(*self))
    }

    pub(crate) fn index_name() -> &'static str {
        "year1"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["1y", "y", "year", "yearly", "year1", "yearindex"]
    }
}

impl From<Year1> for u64 {
    #[inline]
    fn from(value: Year1) -> Self {
        value.0 as u64
    }
}

impl From<Day1> for Year1 {
    #[inline]
    fn from(value: Day1) -> Self {
        Self::from(usize::from(Date::from(value).year() - 2009))
    }
}

impl TryFrom<Date> for Year1 {
    type Error = Error;

    #[inline]
    fn try_from(value: Date) -> Result<Self> {
        value.try_into_jiff()?;
        let years = value
            .year()
            .checked_sub(2009)
            .ok_or(Error::UnindexableDate)?;
        u8::try_from(years)
            .map(Self)
            .map_err(|_| Error::UnindexableDate)
    }
}

impl From<Month1> for Year1 {
    #[inline]
    fn from(value: Month1) -> Self {
        Self((usize::from(value) / 12) as u8)
    }
}

#[cfg(feature = "storage")]
impl PrintableIndex for Year1 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
}
