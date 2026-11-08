use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::Height;

#[derive(Deserialize, JsonSchema)]
pub struct HeightParam {
    #[serde(deserialize_with = "bitview_types::de_plain")]
    pub height: Height,
}
