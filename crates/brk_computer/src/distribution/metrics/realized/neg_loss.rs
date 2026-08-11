use brk_traversable::Traversable;
use brk_types::{Cents, Dollars, Height};
use vecdb::LazyVec;

use crate::internal::{LazyPerBlock, Windows};

#[derive(Clone, Traversable)]
pub struct NegRealizedLoss {
    #[traversable(flatten)]
    pub base: LazyVec<Height, Dollars, Height, Cents>,
    pub sum: Windows<LazyPerBlock<Dollars, Cents>>,
}
