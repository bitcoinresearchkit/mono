use bitview_traversable::Traversable;
use brk_types::{Cents, Dollars, Height};
use vecdb::LazyVec;

use bitview_compute::{LazyPerBlock, Windows};

#[derive(Clone, Traversable)]
pub struct NegRealizedLoss {
    #[traversable(flatten)]
    /// Negative realized loss for the represented block.
    pub base: LazyVec<Height, Dollars, Height, Cents>,
    /// Sum of negative realized loss over the named trailing window.
    pub sum: Windows<LazyPerBlock<Dollars, Cents>>,
}
