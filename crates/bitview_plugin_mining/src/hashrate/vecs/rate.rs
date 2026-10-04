use bitview_primitives::{Hashrate, PartsPerMillionSigned32};
use bitview_traversable::Traversable;
use bitview_vecs::{FixedRatioPerBlock, PerBlock};
use vecdb::{Rw, StorageMode};

use super::HashRateSmaVecs;

#[derive(Traversable)]
pub struct RateVecs<M: StorageMode = Rw> {
    /// Network hash-rate estimate for the represented block, in hashes per
    /// second.
    pub base: PerBlock<Hashrate, M>,
    /// Arithmetic mean of the per-block network hash-rate estimates over a
    /// trailing duration; every block has equal weight.
    pub sma: HashRateSmaVecs<M>,
    /// Running all-time high of the estimated network hash rate, in hashes per
    /// second.
    pub ath: PerBlock<Hashrate, M>,
    /// Estimated network hash rate divided by its running all-time high, minus
    /// one. Zero marks an all-time high; negative values measure the drawdown
    /// below it.
    pub drawdown: FixedRatioPerBlock<PartsPerMillionSigned32, M>,
}
