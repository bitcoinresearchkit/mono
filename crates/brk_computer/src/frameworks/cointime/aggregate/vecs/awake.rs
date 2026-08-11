use brk_traversable::Traversable;
use brk_types::{Cents, StoredF64};

use crate::internal::{
    LazyFiatPerBlock, LazyPerBlock, LazyPriceWithRatioPerBlock, LazySpotValuePerBlock,
};

#[derive(Clone, Traversable)]
pub struct AwakeVecs {
    pub supply: LazySpotValuePerBlock,
    #[traversable(wrap = "supply/in_loss", rename = "share")]
    pub supply_in_loss_share: LazyPerBlock<StoredF64>,
    pub cap: LazyFiatPerBlock<Cents>,
    pub price: LazyPriceWithRatioPerBlock,
}
