use schemars::JsonSchema;
use serde::Deserialize;

use crate::{Limit, SeriesName};

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchQuery {
    /// Search query string
    pub q: SeriesName,
    /// Maximum number of results, 100 when omitted
    #[serde(default)]
    pub limit: Limit,
}
