use brk_traversable::Traversable;
use brk_types::{PartsPerMillion32, StoredF32};
use vecdb::{Rw, StorageMode};

use super::price_min_max_vecs::PriceMinMaxVecs;
use crate::internal::{LazyPerBlock, PerBlock, PercentPerBlock};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    pub min: PriceMinMaxVecs<M>,
    pub max: PriceMinMaxVecs<M>,
    /// Absolute difference in cents per BTC between the current and previous
    /// block's spot prices. The first block is zero; this is not an OHLC range.
    pub true_range: LazyPerBlock<StoredF32>,
    /// Sum of `price_true_range` over the trailing 14-day monotonic-time window,
    /// in cents per BTC. This measures the spot-price path length over the
    /// window, not its high-low range.
    pub true_range_sum_2w: PerBlock<StoredF32, M>,
    pub choppiness_index_2w: PercentPerBlock<PartsPerMillion32, M>,
}
