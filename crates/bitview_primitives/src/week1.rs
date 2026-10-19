use brk_error::{Error, Result};
use brk_types::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Date, Day1};

#[cfg(feature = "storage")]
use vecdb::{Pco, PrintableIndex};

/// Index of a week.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Week1(u16);

brk_types::impl_integer_index!(Week1, u16);

impl Week1 {
    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::from(Date::from(*self))
    }

    pub(crate) fn index_name() -> &'static str {
        "week1"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["1w", "w", "week", "weekly", "week1", "weekindex"]
    }
}

impl From<Day1> for Week1 {
    #[inline]
    fn from(value: Day1) -> Self {
        Self(((usize::from(value) + 3) / 7) as u16)
    }
}

impl TryFrom<Date> for Week1 {
    type Error = Error;

    #[inline]
    fn try_from(value: Date) -> Result<Self> {
        // ISO week 2009-W01 starts three days before the daily epoch.
        let days = value
            .try_into_jiff()?
            .duration_since(Date::JIFF_INDEX_ZERO)
            .as_secs()
            / 86_400
            + 3;
        let days = u32::try_from(days).map_err(|_| Error::UnindexableDate)?;
        u16::try_from(days / 7)
            .map(Self)
            .map_err(|_| Error::UnindexableDate)
    }
}

#[cfg(feature = "storage")]
impl PrintableIndex for Week1 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
}
