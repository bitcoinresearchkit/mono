use schemars::JsonSchema;
use serde::Deserialize;

use crate::PageSize;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchQuery {
    /// Words to find in series names and descriptions (`realized price`, `mvrv`), in any order
    #[schemars(example = &"realized price")]
    pub q: String,
    /// Maximum number of results, 1 to 1,000 (default: 100)
    #[serde(default = "SearchQuery::default_limit")]
    #[schemars(example = 10, example = 100)]
    pub limit: PageSize,
}

impl SearchQuery {
    pub const DEFAULT_LIMIT: PageSize = PageSize::new(100);

    fn default_limit() -> PageSize {
        Self::DEFAULT_LIMIT
    }
}
