use bitview_collections::Windows;
use bitview_traversable::Traversable;
use bitview_vecs::LazyPerBlock;
use brk_types::{Cents, Dollars, Height};
use vecdb::LazyVec;

#[derive(Clone, Traversable)]
pub struct NegRealizedLoss {
    #[traversable(flatten)]
    /// Negative realized loss for the represented block.
    pub base: LazyVec<Height, Dollars, Height, Cents>,
    /// Sum of negative realized loss over each supported trailing window.
    pub sum: Windows<LazyPerBlock<Dollars, Cents>>,
}
