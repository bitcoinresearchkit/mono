use bitview_compute::NumericValue;
use bitview_transforms::Convert;
use brk_types::{Dollars, Height, Version};
use vecdb::{Ident, ReadableCloneableVec};

use crate::{Fiat, FiatType, IndexSources, LazyPerBlock};

/// Lazy fiat: both cents and usd are lazy views of a stored source.
/// Zero extra stored vecs.
pub type LazyFiatPerBlock<C> = Fiat<LazyPerBlock<C>, LazyPerBlock<Dollars, C>>;

impl<C: FiatType> LazyFiatPerBlock<C> {
    pub(crate) fn from_lazy(name: &str, version: Version, source: &LazyPerBlock<C>) -> Self
    where
        C: NumericValue,
    {
        let cents = LazyPerBlock::from_lazy::<Ident, C>(&format!("{name}_cents"), version, source);
        let usd = LazyPerBlock::from_lazy::<Convert, C>(name, version, source);
        Self { usd, cents }
    }

    pub fn from_cents_source(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, C> + ?Sized),
        indexes: &IndexSources,
    ) -> Self
    where
        C: NumericValue,
    {
        let source = LazyPerBlock::from_height_source::<Ident>(
            &format!("{name}_cents"),
            version,
            source,
            indexes,
        );
        Self::from_lazy(name, version, &source)
    }
}
