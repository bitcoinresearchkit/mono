use brk_error::{Error, Result};
use brk_types::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Date, INDEX_EPOCH};

#[cfg(feature = "storage")]
use vecdb::{Pco, PrintableIndex};

const DAY3_INTERVAL: u32 = 259200;

/// Index of a 3-day period.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Day3(u16);

brk_types::impl_integer_index!(Day3, u16);

impl Day3 {
    pub fn from_timestamp(ts: Timestamp) -> Self {
        Self(((*ts).saturating_sub(INDEX_EPOCH - 86400) / DAY3_INTERVAL) as u16)
    }

    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::new(INDEX_EPOCH - 86400 + self.0 as u32 * DAY3_INTERVAL)
    }
}

impl TryFrom<Date> for Day3 {
    type Error = Error;

    fn try_from(date: Date) -> Result<Self> {
        // Bucket zero starts one day before the daily index epoch.
        let days = Date::INDEX_ZERO_.until(date.try_into_jiff()?)?.get_days() + 1;
        let days = u32::try_from(days).map_err(|_| Error::UnindexableDate)?;
        u16::try_from(days / 3)
            .map(Self)
            .map_err(|_| Error::UnindexableDate)
    }
}

impl Day3 {
    pub(crate) fn index_name() -> &'static str {
        "day3"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["3d", "day3"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for Day3 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
}
