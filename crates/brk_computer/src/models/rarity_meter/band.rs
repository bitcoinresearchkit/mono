use brk_traversable::Traversable;
use brk_types::{Cents, PartsPerMillion32, RarityPercentileId};

use crate::internal::{LazyColumnRatioPerBlock, LazyPerBlock, Price};

#[derive(Clone, Traversable)]
pub struct Band {
    #[traversable(flatten)]
    pub ratio: LazyColumnRatioPerBlock<PartsPerMillion32, RarityPercentileId>,
    pub price: Price<LazyPerBlock<Cents>>,
}
