use bitview_types::Cohort;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct UrpdCohortParam {
    pub cohort: Cohort,
}
