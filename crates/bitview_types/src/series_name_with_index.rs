use bitview_primitives::Index;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use bitview_primitives::SeriesName;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct SeriesNameWithIndex {
    pub series: SeriesName,
    pub index: Index,
}
