use std::borrow::Cow;

use bitview_primitives::SeriesName;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A paginated list of available series names (up to 1,000 per page)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PaginatedSeries {
    /// Current page number (0-indexed)
    #[schemars(example = 0)]
    pub current_page: usize,
    /// Maximum valid page index (0-indexed)
    #[schemars(example = 21)]
    pub max_page: usize,
    pub total_count: usize,
    pub per_page: usize,
    pub has_more: bool,
    #[schemars(with = "Vec<SeriesName>")]
    pub series: Vec<Cow<'static, str>>,
}
