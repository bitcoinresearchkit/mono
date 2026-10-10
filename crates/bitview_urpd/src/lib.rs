//! URPD reconstruction and derived metrics from the canonical UTXO set history.
mod metrics;
mod projected_bucket;
mod projection;

use brk_types::Version;

/// Computation revision shared by consumers of weighted URPD buckets.
pub const COMPUTE_VERSION: Version = Version::ONE;

/// Rounding precision for UTXO cost basis prices (5 significant digits in dollars).
pub const COST_BASIS_PRICE_DIGITS: i32 = 5;
pub use metrics::{CostBasisVecs, compute_cost_basis};
pub use projected_bucket::ProjectedBucket;

mod origin_urpd;
pub use origin_urpd::OriginUrpd;

mod replay;
mod replay_inputs;
mod replay_state;
pub use replay::Replay;
pub use replay_inputs::ReplayInputs;
