use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::Index;

use crate::{DataRangeFormat, SeriesList, SeriesName, SeriesSelectionLegacy};

with_range_format! {
    /// Selection of series to query
    #[derive(Debug, Deserialize, JsonSchema)]
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
        Self {
            index,
            series: SeriesList::from(series),
            start: range.start(),
            end: range.end(),
            limit: range.limit(),
            format: range.format(),
        }
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

impl From<SeriesSelectionLegacy> for SeriesSelection {
    #[inline]
    fn from(value: SeriesSelectionLegacy) -> Self {
        let start = value.start();
        let end = value.end();
        let limit = value.limit();
        let format = value.format();

        Self {
            index: value.index,
            series: value.ids,
            start,
            end,
            limit,
            format,
        }
    }
}
