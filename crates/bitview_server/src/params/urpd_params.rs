use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::{Cohort, Date, UrpdAggregation, UrpdWeight};

/// Path parameters for `/api/urpd/{cohort}/{date}`.
#[derive(Deserialize, JsonSchema)]
pub struct UrpdParams {
    pub cohort: Cohort,
    /// Calendar date of the URPD snapshot in `YYYY-MM-DD` format.
    #[schemars(with = "String", example = &"2024-01-01")]
    pub date: Date,
}

/// Path parameters for per-cohort URPD endpoints.
#[derive(Deserialize, JsonSchema)]
pub struct UrpdCohortParam {
    pub cohort: Cohort,
}

/// Query parameters for URPD endpoints.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UrpdQuery {
    /// Aggregation strategy. Default: raw (no aggregation). Accepts `bucket` as alias.
    #[serde(default, rename = "agg", alias = "bucket")]
    pub aggregation: UrpdAggregation,
    /// Supply weighting. Default: raw (unweighted).
    #[serde(default)]
    pub weight: UrpdWeight,
}

/// Query parameters for URPD date discovery.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UrpdWeightQuery {
    /// Supply weighting. Default: raw (unweighted).
    #[serde(default)]
    pub weight: UrpdWeight,
}
