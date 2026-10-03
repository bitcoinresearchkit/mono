use bitview_collections::Windows;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{Ident, ReadableCloneableVec};

use crate::{
    FiatType, IndexSources, LazyFiatBlock, LazyFiatPerBlock, LazyPerBlock,
    LazyRollingSumsFiatFromHeight,
};

#[derive(Clone, Traversable)]
pub struct LazyFiatPerBlockCumulativeWithSums<C: FiatType> {
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    block: LazyFiatBlock<C>,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub(crate) cumulative: LazyFiatPerBlock<C>,
    pub sum: LazyRollingSumsFiatFromHeight<C>,
}

impl<C: FiatType> LazyFiatPerBlockCumulativeWithSums<C> {
    pub fn from_cumulative_cents_source(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, C> + ?Sized),
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Self {
        let source = LazyPerBlock::from_height_source::<Ident>(
            &format!("{name}_cumulative_cents"),
            version,
            source,
            indexes,
        );
        let cumulative =
            LazyFiatPerBlock::from_lazy(&format!("{name}_cumulative"), version, &source);
        let block = LazyFiatBlock::from_cumulative_source(name, version, &source);
        let sum = LazyRollingSumsFiatFromHeight::new(
            &format!("{name}_sum"),
            version,
            &source.height,
            window_starts,
            indexes,
        );
        Self {
            block,
            cumulative,
            sum,
        }
    }
}
