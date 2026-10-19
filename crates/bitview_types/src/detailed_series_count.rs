use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::SeriesCount;

/// Detailed series count with per-database breakdown.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DetailedSeriesCount {
    #[serde(flatten)]
    pub total: SeriesCount,
    pub by_db: BTreeMap<String, SeriesCount>,
}
