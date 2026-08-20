use schemars::JsonSchema;
use serde::Deserialize;

with_range_format! {
    /// Range parameters with output format for API query parameters.
    #[derive(Default, Debug, Deserialize, JsonSchema)]
    #[serde(deny_unknown_fields)]
    pub struct DataRangeFormat {}
}
