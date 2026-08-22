use bitview_traversable::Traversable;
use brk_types::{PartsPerMillion32, StoredF32};
use vecdb::{Rw, StorageMode};

use super::price_min_max_vecs::PriceMinMaxVecs;
use bitview_compute::{LazyPerBlock, PerBlock, PercentPerBlock};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Minimum block-level Bitcoin spot price in a trailing monotonic-time
    /// window, including the represented block.
    pub min: PriceMinMaxVecs<M>,
    /// Maximum block-level Bitcoin spot price in a trailing monotonic-time
    /// window, including the represented block.
    pub max: PriceMinMaxVecs<M>,
    /// Absolute difference in cents per BTC between the represented and
    /// previous blocks' spot prices. The genesis block is zero; this is not an
    /// open-high-low-close price range.
    pub true_range: LazyPerBlock<StoredF32>,
    /// Sum of per-block spot-price true range over the trailing 14-day
    /// monotonic-time window, in cents per BTC. This measures the spot-price
    /// path length over the window, not its high-low range.
    pub true_range_sum_2w: PerBlock<StoredF32, M>,
    /// Two-week Choppiness Index: base-10 logarithm of the trailing true-range
    /// sum divided by the two-week high-low range, divided by the base-10
    /// logarithm of the number of blocks in the window. Returns zero when the
    /// high-low range is zero or the window contains fewer than two blocks.
    /// Higher values mean price traveled a longer path relative to its net
    /// range and was therefore choppier; lower nonzero values mean a more
    /// direct trend.
    pub choppiness_index_2w: PercentPerBlock<PartsPerMillion32, M>,
}
