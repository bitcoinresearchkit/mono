use schemars::JsonSchema;
use serde::Deserialize;

use crate::Limit;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchQuery {
    /// Words to find in series names and descriptions (`realized price`, `mvrv`), in any order
    #[schemars(example = &"realized price")]
    pub q: String,
    /// Maximum number of results, 100 when omitted
    #[serde(default)]
    pub limit: Limit,
}
