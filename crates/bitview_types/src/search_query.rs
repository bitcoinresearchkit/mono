use schemars::JsonSchema;
use serde::Deserialize;

use crate::{Limit, SeriesName};

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchQuery {
    /// Search query string
    pub q: SeriesName,
    /// Maximum number of results
    #[serde(default)]
    pub limit: Limit,
}
