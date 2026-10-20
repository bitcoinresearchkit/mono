use bitview_types::{UrpdAggregation, UrpdWeight};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UrpdQuery {
    /// Aggregation strategy. Default: raw (no aggregation). Accepts `bucket` as alias.
    #[serde(default, rename = "agg", alias = "bucket")]
    pub aggregation: UrpdAggregation,
    /// Supply weighting. Default: none (unweighted).
    #[serde(default)]
    pub weight: UrpdWeight,
}
