use bitview_primitives::{Days, PartsPerMillionSigned32, Seconds};
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
    /// Null while the all-time high is zero.
    pub drawdown: LazyPercentPerBlock<PartsPerMillionSigned32>,
    /// Exact elapsed seconds preserve the ATH timestamp across incremental updates.
    #[traversable(hidden)]
    pub(super) seconds_since: PerBlock<Seconds, M>,
    /// Fractional days, using monotonic block time, since the latest block whose
    /// spot price equaled the running all-time high. Resets to zero at equality.
    pub days_since: LazyPerBlock<Days, Seconds>,
    /// Longest fractional-day interval between consecutive Bitcoin spot-price
    /// all-time highs through the represented block, including the ongoing
    /// interval since the latest one.
    pub max_days_between: PerBlock<Days, M>,
}
