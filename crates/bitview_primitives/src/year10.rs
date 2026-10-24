use brk_error::{Error, Result};
use brk_types::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Date, Day1, Year1};

#[cfg(feature = "storage")]
use vecdb::{Pco, PrintableIndex};

/// Index of a decade.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Year10(u8);

brk_types::impl_integer_index!(Year10, u8);

impl Year10 {
    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::from(Date::from(*self))
    }

    pub(crate) fn index_name() -> &'static str {
        "year10"
    }
}

impl From<Day1> for Year10 {
    #[inline]
    fn from(value: Day1) -> Self {
        Self::from(usize::from((Date::from(value).year() - 2000) / 10))
    }
}

impl TryFrom<Date> for Year10 {
    type Error = Error;

    #[inline]
    fn try_from(value: Date) -> Result<Self> {
        value.try_into_jiff()?;
        let years = value
            .year()
            .checked_sub(2000)
            .ok_or(Error::UnindexableDate)?;
        u8::try_from(years / 10)
            .map(Self)
            .map_err(|_| Error::UnindexableDate)
    }
}

impl From<Year1> for Year10 {
    #[inline]
    fn from(value: Year1) -> Self {
        let v = usize::from(value);
        if v == 0 {
            Self(0)
        } else {
            Self((((v - 1) / 10) + 1) as u8)
        }
    }
}

#[cfg(feature = "storage")]
impl PrintableIndex for Year10 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
}
