use bitview_traversable::Traversable;
use brk_types::{Cents, StoredF64};

use bitview_compute::{
    LazyFiatPerBlock, LazyPerBlock, LazyPriceWithRatioPerBlock, LazySpotValuePerBlock,
};

#[derive(Clone, Traversable)]
pub struct AwakeVecs {
    /// Sum of supply multiplied by wakefulness across the selected UTXO age
    /// ranges. Each age-range contribution is rounded down to whole satoshis.
    pub supply: LazySpotValuePerBlock,
    /// Share of awake supply that is in loss: the sum of supply in loss
    /// multiplied by wakefulness divided by the sum of total supply multiplied
    /// by wakefulness. Returns NaN when the weighted supply is zero.
    #[traversable(wrap = "supply/in_loss", rename = "share")]
    pub supply_in_loss_share: LazyPerBlock<StoredF64>,
    /// Sum of realized capitalization multiplied by wakefulness across the
    /// selected UTXO age ranges.
    pub cap: LazyFiatPerBlock<Cents>,
    /// Awake capitalization divided by awake supply in BTC. Returns zero when
    /// awake supply is zero.
    pub price: LazyPriceWithRatioPerBlock,
}
