use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::Height;

#[derive(Deserialize, JsonSchema)]
pub struct StartParam {
    /// The first block's height.
    pub start: Height,
}
