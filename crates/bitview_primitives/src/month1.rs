use brk_error::{Error, Result};
use brk_types::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Date, Day1};

#[cfg(feature = "storage")]
use vecdb::{Pco, PrintableIndex};

/// Index of a month.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Month1(u16);

brk_types::impl_integer_index!(Month1, u16);

impl Month1 {
    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::from(Date::from(*self))
    }
}

impl From<Month1> for u64 {
    #[inline]
    fn from(value: Month1) -> Self {
        value.0 as u64
    }
}

impl From<Day1> for Month1 {
    #[inline]
    fn from(value: Day1) -> Self {
        let date = Date::from(value);
        Self::from(usize::from(date.year() - 2009) * 12 + usize::from(date.month()) - 1)
    }
}

impl TryFrom<Date> for Month1 {
    type Error = Error;

    #[inline]
    fn try_from(value: Date) -> Result<Self> {
        value.try_into_jiff()?;
        let years = value
            .year()
            .checked_sub(2009)
            .ok_or(Error::UnindexableDate)?;
        let months = u32::from(years) * 12 + u32::from(value.month()) - 1;
        u16::try_from(months)
            .map(Self)
            .map_err(|_| Error::UnindexableDate)
    }
}

impl Month1 {
    pub(crate) fn index_name() -> &'static str {
        "month1"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["month", "m", "monthly", "month1", "monthindex", "1m", "1mo"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for Month1 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
}
