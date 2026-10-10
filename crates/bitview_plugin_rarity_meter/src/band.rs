use bitview_primitives::PartsPerMillion32;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, LazyRatioPerBlock, Price};
use brk_types::Cents;

#[derive(Clone, Traversable)]
pub struct Band {
    #[traversable(flatten)]
    pub ratio: LazyRatioPerBlock<PartsPerMillion32>,
    /// The reference price times the historical ratio percentile.
    pub band: Price<LazyPerBlock<Cents>>,
}
