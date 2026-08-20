use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use brk_types::Index;

/// Metadata about a series
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SeriesInfo {
    /// Human-readable metric definition, when documented
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Cow<'static, str>>,
    /// Available indexes
    pub indexes: Vec<Index>,
    /// Value type (e.g. "f32", "u64", "Sats")
    #[serde(rename = "type")]
    pub value_type: Cow<'static, str>,
}
