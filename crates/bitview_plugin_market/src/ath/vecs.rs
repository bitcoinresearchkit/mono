use bitview_primitives::{PartsPerMillionSigned32, Seconds, StoredF32};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, LazyPercentPerBlock, PerBlock, Price};
use brk_types::Cents;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Running all-time high of the Bitcoin spot price through the represented
    /// block.
    pub high: Price<PerBlock<Cents, M>>,
    /// Bitcoin spot price divided by its running all-time high, minus one. Zero
    /// marks an all-time high; negative values measure the drawdown below it.
    pub drawdown: LazyPercentPerBlock<PartsPerMillionSigned32>,
    /// Exact elapsed seconds preserve the ATH timestamp across incremental updates.
    #[traversable(hidden)]
    pub(super) seconds_since: PerBlock<Seconds, M>,
    /// Fractional days, using monotonic block time, since the latest block whose
    /// spot price equaled the running all-time high. Resets to zero at equality.
    pub days_since: LazyPerBlock<StoredF32, Seconds>,
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
