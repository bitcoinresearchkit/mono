use bitview_traversable::Traversable;
use brk_types::{Cents, PartsPerMillionSigned32, StoredF32};
use vecdb::{Rw, StorageMode};

use bitview_compute::{LazyPerBlock, LazyPercentPerBlock, PerBlock, Price};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Running all-time high of the Bitcoin spot price through the current
    /// block.
    pub high: Price<PerBlock<Cents, M>>,
    /// Bitcoin spot price divided by its running all-time high, minus one.
    pub drawdown: LazyPercentPerBlock<PartsPerMillionSigned32>,
    /// Fractional days, using monotonic block time, since the latest block whose
    /// spot price equaled the running all-time high. Resets to zero at equality.
    pub days_since: PerBlock<StoredF32, M>,
    /// `days_since_price_ath` divided by 365.
    pub years_since: LazyPerBlock<StoredF32>,
    /// Running maximum of `days_since_price_ath`, including the current
    /// unfinished interval between all-time highs.
    pub max_days_between: PerBlock<StoredF32, M>,
    /// `max_days_between_price_ath` divided by 365.
    pub max_years_between: LazyPerBlock<StoredF32>,
}
