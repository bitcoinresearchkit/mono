//! Calculation algorithms shared by Bitview plugins.
//!
//! Metric vector ownership and view composition live in bitview_vecs.
mod cohort_accounting;
mod cohort_weight;
mod drawdown;
mod prepare;
mod statistics;
mod traits;
mod weighted;

pub use cohort_accounting::{CohortAccounting, collect_age_range};
pub use cohort_weight::{collect_cohort_weights, resolve_cohort_weight};
pub use drawdown::ComputeDrawdown;
pub use prepare::{
    Inputs, WRITE_INTERVAL, prepare_computed, save_stamped, write_chunks, write_locked,
};
pub use statistics::{
    AverageWindow, ComputeRollingMedianFromStarts, ComputeRollingStats, DistributionStats,
    ExactOrderStats, ExtremaWindow, FenwickNode, FenwickTree, Retained, SortedWindow,
    compute_rolling_distribution_from_starts, compute_rolling_extrema_from_starts,
    lower_nearest_rank,
};
pub use traits::{ComputedVecValue, FixedRatio, NumericValue, Quantity};
pub use weighted::{
    WeightedCapitalizedPrice, WeightedCohortState, WeightedRatio, weighted_age_aggregates,
};
