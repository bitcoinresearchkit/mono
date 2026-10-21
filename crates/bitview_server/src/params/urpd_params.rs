use bitview_types::{HeightOrDate, UrpdCohort};
use schemars::JsonSchema;
use serde::Deserialize;

/// A URPD cohort and the confirmed point to read it at.
#[derive(Deserialize, JsonSchema)]
pub struct UrpdParams {
    pub cohort: UrpdCohort,
    /// Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)
    pub point: HeightOrDate,
}
