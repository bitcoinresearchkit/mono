use bitview_traversable::Traversable;
use brk_types::{PartsPerMillionSigned32, StoredF64};
use vecdb::{Rw, StorageMode};

use super::HashRateSmaVecs;
use bitview_compute::{PerBlock, PercentPerBlock};

#[derive(Traversable)]
pub struct RateVecs<M: StorageMode = Rw> {
    /// Estimated network hash rate in hashes per second: the current block's
    /// difficulty-implied target hash rate multiplied by the number of blocks
    /// in the trailing 24 hours and divided by 144. The current difficulty is
    /// used for the entire estimate.
    pub base: PerBlock<StoredF64, M>,
    /// Arithmetic mean of the per-block network hash-rate estimates over the
    /// trailing duration named by the series; every block has equal weight.
    /// Durations are fixed at 7, 30, 60, or 365 days.
    pub sma: HashRateSmaVecs<M>,
    /// Running all-time high of the estimated network hash rate, in hashes per
    /// second.
    pub ath: PerBlock<StoredF64, M>,
    /// Estimated network hash rate divided by its running all-time high, minus
    /// one.
    pub drawdown: PercentPerBlock<PartsPerMillionSigned32, M>,
}
