use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A paginated list of available series names (1000 per page)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PaginatedSeries {
    /// Current page number (0-indexed)
    #[schemars(example = 0)]
    pub current_page: usize,
    /// Maximum valid page index (0-indexed)
    #[schemars(example = 21)]
    pub max_page: usize,
    /// Total number of series
    pub total_count: usize,
    /// Results per page
    pub per_page: usize,
    /// Whether more pages are available after the current one
    pub has_more: bool,
    /// List of series names
    pub series: Vec<Cow<'static, str>>,
}
