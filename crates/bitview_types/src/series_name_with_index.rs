use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use brk_types::Index;

use crate::SeriesName;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct SeriesNameWithIndex {
    /// Series name
    pub series: SeriesName,

    /// Aggregation index
    pub index: Index,
}

impl SeriesNameWithIndex {
    pub fn new(series: impl Into<SeriesName>, index: Index) -> Self {
        Self {
            series: series.into(),
            index,
        }
    }
}

impl From<(SeriesName, Index)> for SeriesNameWithIndex {
    fn from((series, index): (SeriesName, Index)) -> Self {
        Self { series, index }
    }
}

impl From<(&str, Index)> for SeriesNameWithIndex {
    fn from((series, index): (&str, Index)) -> Self {
        Self {
            series: series.into(),
            index,
        }
    }
}
