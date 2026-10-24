use bitview_primitives::Index;
use schemars::JsonSchema;
use serde::Deserialize;

use bitview_primitives::SeriesName;

use crate::{DataRangeFormat, SeriesList};

with_range_format! {
    #[derive(Clone, Debug, Deserialize, JsonSchema)]
    #[serde(deny_unknown_fields)]
    pub struct SeriesSelection {
        pub series: SeriesList,

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
            start: range.start,
            end: range.end,
            limit: range.limit,
            format: range.format,
        }
    }
}
