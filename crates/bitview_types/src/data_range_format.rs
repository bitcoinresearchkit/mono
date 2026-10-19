use schemars::JsonSchema;
use serde::Deserialize;

with_range_format! {
    #[derive(Default, Debug, Deserialize, JsonSchema)]
    #[serde(deny_unknown_fields)]
    pub struct DataRangeFormat {}
}
