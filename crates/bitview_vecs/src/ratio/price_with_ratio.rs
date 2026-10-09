use bitview_primitives::PriceRatio;
use bitview_traversable::Traversable;
use derive_more::{Deref, DerefMut};

use crate::LazyRatioPerBlock;

/// A price family and its lazy spot/reference ratio, sharing their existing sources.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct PriceWithRatio<P> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten, rename = "usd")]
    pub(crate) price: P,
    #[traversable(flatten)]
    pub relative: LazyRatioPerBlock<PriceRatio>,
}
