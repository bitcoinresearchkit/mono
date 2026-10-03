use bitview_types::Cohort;
use schemars::JsonSchema;
use serde::Deserialize;

/// Path parameters for per-cohort URPD endpoints.
#[derive(Deserialize, JsonSchema)]
pub struct UrpdCohortParam {
    pub cohort: Cohort,
}
