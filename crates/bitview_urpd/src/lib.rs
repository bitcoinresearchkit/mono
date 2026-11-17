//! URPD reconstruction and derived metrics from the canonical UTXO set history.
mod block_changes;
mod density_band;
mod metrics;
mod price_place;
mod projected_bucket;
mod projection;

use brk_types::Version;

/// Computation revision shared by consumers of weighted URPD buckets.
pub const COMPUTE_VERSION: Version = Version::TWO;

/// Rounding precision for UTXO cost basis prices: 4 significant digits of the price in cents (exact
/// cents under $100, steps of at most 0.1% above, so at most 0.05% off). Five digits made the
/// models' full recompute 3.3x slower: early blocks each got their own price slot.
pub const COST_BASIS_PRICE_DIGITS: u32 = 4;
pub use density_band::DensityBand;
pub use metrics::{CostBasisVecs, compute_cost_basis};
pub use projected_bucket::ProjectedBucket;

mod urpd_history;
mod urpd_walk;
pub use urpd_history::{UrpdCells, UrpdHistory};
pub use urpd_walk::UrpdWalk;
