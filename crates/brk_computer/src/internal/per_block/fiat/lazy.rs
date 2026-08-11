use brk_traversable::Traversable;
use brk_types::{Dollars, Height, Version};
use vecdb::ReadableBoxedVec;

use crate::{
    indexes,
    internal::{FiatType, Identity, LazyPerBlock, NumericValue},
};

/// Lazy fiat: both cents and usd are lazy views of a stored source.
/// Zero extra stored vecs.
#[derive(Clone, Traversable)]
pub struct LazyFiatPerBlock<C: FiatType> {
    pub usd: LazyPerBlock<Dollars, C>,
    pub cents: LazyPerBlock<C, C>,
}

impl<C: FiatType> LazyFiatPerBlock<C> {
    pub(crate) fn from_lazy(name: &str, version: Version, source: &LazyPerBlock<C>) -> Self
    where
        C: NumericValue,
    {
        let cents =
            LazyPerBlock::from_lazy::<Identity<C>, C>(&format!("{name}_cents"), version, source);
        let usd = LazyPerBlock::from_lazy::<C::ToDollars, C>(name, version, source);
        Self { usd, cents }
    }

    pub(crate) fn from_boxed_cents_source(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Height, C>,
        indexes: &indexes::Vecs,
    ) -> Self
    where
        C: NumericValue,
    {
        let source = LazyPerBlock::from_uncached_boxed_height_source::<Identity<C>>(
            &format!("{name}_cents"),
            version,
            source,
            indexes,
        );
        Self::from_lazy(name, version, &source)
    }
}
