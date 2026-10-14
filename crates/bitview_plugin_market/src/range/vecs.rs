use bitview_traversable::Traversable;
use bitview_vecs::LazyFiatPerBlock;
use brk_types::Cents;
use vecdb::{Rw, StorageMode};

use super::price_min_max_vecs::PriceMinMaxVecs;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Minimum block-level Bitcoin spot price in a trailing monotonic-time
    /// window, including the represented block.
    pub min: PriceMinMaxVecs<M>,
    /// Maximum block-level Bitcoin spot price in a trailing monotonic-time
    /// window, including the represented block.
    pub max: PriceMinMaxVecs<M>,
    /// Absolute difference in USD per BTC between the represented and
    /// previous blocks' spot prices. The genesis block is zero; this is not an
    /// open-high-low-close price range.
    pub true_range: LazyFiatPerBlock<Cents>,
}
