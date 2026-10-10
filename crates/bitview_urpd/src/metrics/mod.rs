//! Derive, store and expose per-block URPD metrics.
pub(super) mod bounds;
mod compute;
mod cost_basis;
mod metric_buckets;

pub use compute::compute_cost_basis;
pub use cost_basis::CostBasisVecs;

const WRITE_INTERVAL_BLOCKS: usize = 10_000;
