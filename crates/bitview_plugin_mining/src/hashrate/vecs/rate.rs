use bitview_traversable::Traversable;
use brk_types::{PartsPerMillionSigned32, StoredF64};
use vecdb::{Rw, StorageMode};

use super::HashRateSmaVecs;
use bitview_compute::{PerBlock, PercentPerBlock};

#[derive(Traversable)]
pub struct RateVecs<M: StorageMode = Rw> {
    /// Network hash-rate estimate for the represented block, in hashes per
    /// second.
    pub base: PerBlock<StoredF64, M>,
    /// Arithmetic mean of the per-block network hash-rate estimates over a
    /// trailing duration; every block has equal weight.
    pub sma: HashRateSmaVecs<M>,
    /// Running all-time high of the estimated network hash rate, in hashes per
    /// second.
    pub ath: PerBlock<StoredF64, M>,
    /// Estimated network hash rate divided by its running all-time high, minus
    /// one. Zero marks an all-time high; negative values measure the drawdown
    /// below it.
    pub drawdown: PercentPerBlock<PartsPerMillionSigned32, M>,
}
