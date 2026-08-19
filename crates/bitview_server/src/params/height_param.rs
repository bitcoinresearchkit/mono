use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::Height;

/// Block height path parameter
#[derive(Deserialize, JsonSchema)]
pub struct HeightParam {
    pub height: Height,
}
