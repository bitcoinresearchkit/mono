//! URPD reconstruction and derived metrics from canonical UTXO history.
mod age_ranges;
mod distribution;
mod metrics;
mod origins;
mod projected_bucket;
mod response;

use brk_types::Version;

/// Computation revision shared by consumers of weighted URPD buckets.
pub const COMPUTE_VERSION: Version = Version::ONE;

pub use age_ranges::AgeRangeUrpds;
pub use distribution::{
    AgeCutoffs, COST_BASIS_PRICE_DIGITS, accumulate_masses, collect_mass, rounded_entries,
    weighted_entries,
};
pub use metrics::{Metrics, bounds::AgeBoundsMetrics};
pub use projected_bucket::ProjectedBucket;
pub use response::build_response;

mod origin_urpd;
pub use origin_urpd::OriginUrpd;

mod replay;
mod replay_inputs;
mod replay_state;
pub use replay::Replay;
pub use replay_inputs::ReplayInputs;
