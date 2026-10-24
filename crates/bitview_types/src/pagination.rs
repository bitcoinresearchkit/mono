use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::PageSize;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Pagination {
    /// Zero-based page number (default: 0)
    #[serde(default)]
    #[schemars(example = 0, example = 1, example = 2)]
    pub page: usize,
    /// Results per page, 1 to 1,000 (default: 1000)
    #[serde(default = "Pagination::default_per_page")]
    #[schemars(example = 100, example = 1000)]
    pub per_page: PageSize,
}

impl Pagination {
    pub const DEFAULT_PER_PAGE: PageSize = PageSize::new(PageSize::MAX);

    fn default_per_page() -> PageSize {
        Self::DEFAULT_PER_PAGE
    }

    pub fn page(&self) -> usize {
        self.page
    }

    pub fn per_page(&self) -> usize {
        *self.per_page
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
