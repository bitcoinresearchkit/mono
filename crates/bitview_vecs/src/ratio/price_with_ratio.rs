use bitview_traversable::Traversable;
use brk_types::PriceRatio;
use derive_more::{Deref, DerefMut};

use crate::LazyRatioPerBlock;

/// A price family and its lazy spot/reference ratio, sharing their existing sources.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct PriceWithRatio<P> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub(crate) price: P,
    #[traversable(flatten)]
    pub relative: LazyRatioPerBlock<PriceRatio>,
}
