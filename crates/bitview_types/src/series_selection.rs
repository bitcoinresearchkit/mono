use bitview_primitives::Index;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{DataRangeFormat, SeriesList, SeriesName};

with_range_format! {
    /// Selection of series to query
    #[derive(Clone, Debug, Deserialize, JsonSchema)]
    #[serde(deny_unknown_fields)]
    pub struct SeriesSelection {
        /// Requested series
        #[serde(alias = "m", alias = "metrics")]
        pub series: SeriesList,

        /// Index to query
        #[serde(alias = "i")]
        pub index: Index,
    }
}

impl From<(Index, SeriesName, DataRangeFormat)> for SeriesSelection {
    #[inline]
    fn from((index, series, range): (Index, SeriesName, DataRangeFormat)) -> Self {
        Self::from((index, SeriesList::from(series), range))
    }
}

impl From<(Index, SeriesList, DataRangeFormat)> for SeriesSelection {
    #[inline]
    fn from((index, series, range): (Index, SeriesList, DataRangeFormat)) -> Self {
        Self {
            index,
            series,
            start: range.start(),
            end: range.end(),
            limit: range.limit(),
            format: range.format(),
        }
    }
}
