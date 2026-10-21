use bitview_types::UrpdCohort;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct UrpdCohortParam {
    pub cohort: UrpdCohort,
}
