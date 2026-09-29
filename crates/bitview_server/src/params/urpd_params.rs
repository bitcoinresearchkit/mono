use brk_types::Cohort;
use schemars::JsonSchema;
use serde::Deserialize;

/// A URPD cohort and exact block height or UTC calendar-day alias.
#[derive(Deserialize, JsonSchema)]
pub struct UrpdParams {
    pub cohort: Cohort,
    #[schemars(example = &"840000")]
    pub point: String,
}
