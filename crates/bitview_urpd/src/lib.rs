//! URPD reconstruction and derived metrics from the canonical UTXO set history.
mod density_band;
mod metrics;
mod price_place;
mod projected_bucket;
mod projection;

use brk_types::Version;

/// Computation revision shared by consumers of weighted URPD buckets.
pub const COMPUTE_VERSION: Version = Version::TWO;

pub use bitview_types::COST_BASIS_PRICE_DIGITS;
pub use density_band::DensityBand;
pub use metrics::{CostBasisVecs, compute_cost_basis};
pub use projected_bucket::ProjectedBucket;

mod urpd_history;
mod urpd_walk;
pub use urpd_history::{UrpdCells, UrpdHistory};
pub use urpd_walk::UrpdWalk;
