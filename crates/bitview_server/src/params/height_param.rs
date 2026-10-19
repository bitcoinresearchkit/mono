use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::Height;

#[derive(Deserialize, JsonSchema)]
pub struct HeightParam {
    pub height: Height,
}
