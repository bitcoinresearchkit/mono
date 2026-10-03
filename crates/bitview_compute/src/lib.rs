//! Calculation algorithms shared by Bitview plugins.
//!
//! Metric vector ownership and view composition live in bitview_vecs.
mod age_band;
mod cohort_accounting;
mod cohort_weight;
mod drawdown;
mod prepare;
mod statistics;
mod traits;
mod weighted;

pub use age_band::{AgeBand, MINIMUM_DURATION_DAYS};
pub use cohort_accounting::{CohortAccounting, collect_age_range};
pub use cohort_weight::{collect_cohort_weights, resolve_cohort_weight};
pub use drawdown::ComputeDrawdown;
pub use prepare::prepare_computed;
pub use statistics::{
    ComputeRollingMedianFromStarts, ExactOrderStats, FenwickNode, FenwickTree,
    compute_rolling_distribution_from_starts, compute_rolling_extrema_from_starts,
};
pub use traits::{ComputedVecValue, FixedRatio, NumericValue};
pub use weighted::{
    WeightedCapitalizedPrice, WeightedCohortAggregates, WeightedCohortState, WeightedRatio,
};
