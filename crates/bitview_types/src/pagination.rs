use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Pagination parameters for paginated API endpoints
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Pagination {
    /// Pagination index
    #[serde(default, alias = "p")]
    #[schemars(example = 0, example = 1, example = 2)]
    pub page: Option<usize>,
    /// Results per page (default: 1000, max: 1000)
    #[serde(default)]
    #[schemars(example = 100, example = 1000)]
    pub per_page: Option<usize>,
}

impl Pagination {
    pub const DEFAULT_PER_PAGE: usize = 1_000;
    pub const MAX_PER_PAGE: usize = 1_000;

    pub fn page(&self) -> usize {
        self.page.unwrap_or_default()
    }

    pub fn per_page(&self) -> usize {
        self.per_page
            .filter(|&n| n > 0)
            .unwrap_or(Self::DEFAULT_PER_PAGE)
            .min(Self::MAX_PER_PAGE)
    }

    pub fn start(&self, len: usize) -> usize {
        self.page().saturating_mul(self.per_page()).min(len)
    }

    pub fn end(&self, len: usize) -> usize {
        (self.page().saturating_add(1))
            .saturating_mul(self.per_page())
            .min(len)
    }
}
