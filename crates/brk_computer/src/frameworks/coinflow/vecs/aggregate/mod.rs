use brk_traversable::Traversable;
use brk_types::{Cents, StoredF64};

mod sources;

pub use sources::AggregateSources;

use crate::internal::{
    LazyFiatPerBlock, LazyPerBlock, LazyPriceWithRatioPerBlock, LazySpotValuePerBlock,
};

use super::{super::Horizons, HorizonVecs, Mobility};

#[derive(Clone, Traversable)]
pub struct AggregateVecs {
    pub supply: Mobility<LazySpotValuePerBlock>,
    #[traversable(wrap = "supply/mobile/in_loss", rename = "share")]
    pub supply_in_loss_share: LazyPerBlock<StoredF64>,
    pub horizon: Horizons<HorizonVecs>,
    pub cap: LazyFiatPerBlock<Cents>,
    pub price: LazyPriceWithRatioPerBlock,
}
