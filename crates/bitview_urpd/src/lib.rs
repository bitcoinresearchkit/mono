//! Shared URPD distributions, derived metrics, and snapshot storage.
mod distribution;
mod metrics;
mod response;
mod snapshots;

use brk_types::Version;

/// Computation revision shared by consumers of weighted URPD buckets.
pub const COMPUTE_VERSION: Version = Version::ONE;

pub use distribution::{
    AgeCutoffs, COST_BASIS_PRICE_DIGITS, DailyUrpds, UrpdRaw, accumulate_masses, collect_mass,
    cost_basis_percentiles, rounded_entries, weighted_entries,
};
pub use metrics::{Metrics, bounds::AgeBoundsMetrics};
pub use response::build_response;
pub use snapshots::{AgeRangeUrpds, EncodedAgeRangeUrpds, prune_snapshots};
