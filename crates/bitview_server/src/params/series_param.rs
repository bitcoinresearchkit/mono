use schemars::JsonSchema;
use serde::Deserialize;

use bitview_primitives::SeriesName;

#[derive(Deserialize, JsonSchema)]
pub struct SeriesParam {
    pub series: SeriesName,
}
