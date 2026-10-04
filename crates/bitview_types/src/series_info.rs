use std::{borrow::Cow, sync::Arc};

use bitview_primitives::Index;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Metadata about a series
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SeriesInfo {
    /// Human-readable metric definition, when documented
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Arc<str>>,
    /// Available indexes
    pub indexes: Vec<Index>,
    /// Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
    /// undefined one (e.g. NaN)
    pub nullable: Vec<Index>,
    /// Value type (e.g. "Ratio", "Sats", "Cents")
    #[serde(rename = "type")]
    pub value_type: Cow<'static, str>,
}
