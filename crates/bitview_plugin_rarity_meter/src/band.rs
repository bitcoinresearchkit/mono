use bitview_primitives::{Multiple, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, LazyRatioPerBlock, Price};
use brk_types::Cents;

#[derive(Clone, Traversable)]
pub struct Band {
    #[traversable(flatten)]
    pub ratio: LazyRatioPerBlock<PartsPerMillion32, Multiple>,
    pub band: Price<LazyPerBlock<Cents>>,
}
