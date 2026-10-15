use bitview_primitives::Index;
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
