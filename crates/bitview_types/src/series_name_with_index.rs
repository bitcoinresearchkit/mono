use brk_types::Index;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::SeriesName;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct SeriesNameWithIndex {
    /// Series name
    pub series: SeriesName,

    /// Aggregation index
    pub index: Index,
}

impl SeriesNameWithIndex {
    fn new(series: impl Into<SeriesName>, index: Index) -> Self {
        Self {
            series: series.into(),
            index,
        }
    }
}

impl From<(SeriesName, Index)> for SeriesNameWithIndex {
    fn from((series, index): (SeriesName, Index)) -> Self {
        Self::new(series, index)
    }
}

impl From<(&str, Index)> for SeriesNameWithIndex {
    fn from((series, index): (&str, Index)) -> Self {
        Self::new(series, index)
    }
}
