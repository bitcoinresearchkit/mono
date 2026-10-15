use brk_error::{Error, Result};
use brk_types::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Date, Month1};

#[cfg(feature = "storage")]
use vecdb::{Pco, PrintableIndex};

/// Index of a quarter (3 months).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Month3(u8);

brk_types::impl_integer_index!(Month3, u8);

impl Month3 {
    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::from(Date::from(*self))
    }
}

impl TryFrom<Date> for Month3 {
    type Error = Error;

    #[inline]
    fn try_from(value: Date) -> Result<Self> {
        let months = u16::from(Month1::try_from(value)?);
        u8::try_from(months / 3)
            .map(Self)
            .map_err(|_| Error::UnindexableDate)
    }
}

impl From<Month1> for Month3 {
    #[inline]
    fn from(value: Month1) -> Self {
        Self((usize::from(value) / 3) as u8)
    }
}

impl Month3 {
    pub(crate) fn index_name() -> &'static str {
        "month3"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &[
            "quarter",
            "q",
            "quarterly",
            "month3",
            "quarterindex",
            "3m",
            "3mo",
        ]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for Month3 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
}
