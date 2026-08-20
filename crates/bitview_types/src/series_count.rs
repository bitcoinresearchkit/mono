use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Series count statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SeriesCount {
    /// Number of unique series available (e.g., realized_price, market_cap)
    #[schemars(example = 3141)]
    pub distinct: usize,
    /// Total number of series-index combinations across all timeframes
    #[schemars(example = 21000)]
    pub total: usize,
    /// Number of lazy (computed on-the-fly) series-index combinations
    #[schemars(example = 5000)]
    pub lazy: usize,
    /// Number of eager (stored on disk) series-index combinations
    #[schemars(example = 16000)]
    pub stored: usize,
}
