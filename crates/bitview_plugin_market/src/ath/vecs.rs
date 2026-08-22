use bitview_traversable::Traversable;
use brk_types::{Cents, PartsPerMillionSigned32, StoredF32};
use vecdb::{Rw, StorageMode};

use bitview_compute::{LazyPerBlock, LazyPercentPerBlock, PerBlock, Price};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Running all-time high of the Bitcoin spot price through the represented
    /// block.
    pub high: Price<PerBlock<Cents, M>>,
    /// Bitcoin spot price divided by its running all-time high, minus one. Zero
    /// marks an all-time high; negative values measure the drawdown below it.
    pub drawdown: LazyPercentPerBlock<PartsPerMillionSigned32>,
    /// Fractional days, using monotonic block time, since the latest block whose
    /// spot price equaled the running all-time high. Resets to zero at equality.
    pub days_since: PerBlock<StoredF32, M>,
    /// Fractional years since the latest Bitcoin spot-price all-time high,
    /// equal to fractional days since that high divided by 365.
    pub years_since: LazyPerBlock<StoredF32>,
    /// Longest fractional-day interval since a Bitcoin spot-price all-time high
    /// observed through the represented block, including the ongoing interval.
    pub max_days_between: PerBlock<StoredF32, M>,
    /// Longest fractional-year interval since a Bitcoin spot-price all-time
    /// high observed through the represented block, equal to the longest
    /// fractional-day interval divided by 365.
    pub max_years_between: LazyPerBlock<StoredF32>,
}
