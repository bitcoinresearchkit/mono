use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::SeriesName;

#[derive(Deserialize, JsonSchema)]
pub struct SeriesParam {
    pub series: SeriesName,
}
