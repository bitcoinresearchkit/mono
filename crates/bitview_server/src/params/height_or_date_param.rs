use bitview_types::HeightOrDate;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct HeightOrDateParam {
    /// Confirmed block height (`840000`) or UTC calendar day (`YYYY-MM-DD`)
    pub point: HeightOrDate,
}
