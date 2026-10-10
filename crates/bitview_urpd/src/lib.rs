//! URPD reconstruction and derived metrics from the canonical UTXO set history.
mod metrics;
mod projected_bucket;
mod projection;

use brk_types::Version;

/// Computation revision shared by consumers of weighted URPD buckets.
pub const COMPUTE_VERSION: Version = Version::TWO;

/// Rounding precision for UTXO cost basis prices: 4 significant digits of the price in cents (exact
/// cents under $100, steps of at most 0.1% above, so at most 0.05% off). Five digits made the
/// models' full recompute 3.3x slower: early blocks each got their own price slot.
pub const COST_BASIS_PRICE_DIGITS: u32 = 4;
pub use metrics::{CostBasisVecs, compute_cost_basis};
pub use projected_bucket::ProjectedBucket;

mod origin_urpd;
pub use origin_urpd::OriginUrpd;

mod replay;
mod replay_inputs;
mod replay_state;
pub use replay::Replay;
pub use replay_inputs::ReplayInputs;
