use std::{borrow::Cow, sync::Arc};

use bitview_primitives::{Accumulation, Index};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SeriesInfo {
    /// Human-readable metric definition, when documented
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Arc<str>>,
    pub indexes: Vec<Index>,
    /// Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
    /// undefined one (e.g. NaN)
    pub nullable: Vec<Index>,
    /// Value type (e.g. "Ratio", "Sats", "Cents")
    #[serde(rename = "type")]
    pub value_type: Cow<'static, str>,
    /// What the value type measures, when documented (e.g. "A duration in days.")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<Arc<str>>,
    /// How values combine over time: a `flow` sums over a window, a `level` is read at a point, and a `cumulative` series is already a running total, its flow the difference.
    pub accumulation: Accumulation,
}
