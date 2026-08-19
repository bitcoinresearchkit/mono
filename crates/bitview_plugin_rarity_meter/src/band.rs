use bitview_traversable::Traversable;
use brk_types::{Cents, PartsPerMillion32, RarityPercentileId};

use bitview_compute::{LazyColumnRatioPerBlock, LazyPerBlock, Price};

#[derive(Clone, Traversable)]
pub struct Band {
    #[traversable(flatten)]
    pub ratio: LazyColumnRatioPerBlock<PartsPerMillion32, RarityPercentileId>,
    pub price: Price<LazyPerBlock<Cents>>,
}
