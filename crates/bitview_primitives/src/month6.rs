use brk_error::{Error, Result};
use brk_types::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Date, Month1};

#[cfg(feature = "storage")]
use vecdb::{Pco, PrintableIndex};

/// Index of a half-year (6 months).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Month6(u8);

brk_types::impl_integer_index!(Month6, u8);

impl Month6 {
    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::from(Date::from(*self))
    }

    pub(crate) fn index_name() -> &'static str {
        "month6"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["semester", "s", "month6", "semesterindex", "6m", "6mo"]
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

#[cfg(feature = "storage")]
impl PrintableIndex for Month6 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
}
