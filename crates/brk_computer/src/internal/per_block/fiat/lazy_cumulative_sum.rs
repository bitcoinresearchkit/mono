use brk_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::ReadableBoxedVec;

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, FiatType, Identity, LazyFiatBlock, LazyFiatPerBlock, LazyPerBlock,
        LazyRollingSumsFiatFromHeight, Windows,
    },
};

#[derive(Clone, Traversable)]
pub struct LazyFiatPerBlockCumulativeWithSums<C: FiatType> {
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub block: LazyFiatBlock<C>,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: LazyFiatPerBlock<C>,
    pub sum: LazyRollingSumsFiatFromHeight<C>,
}

impl<C: FiatType> LazyFiatPerBlockCumulativeWithSums<C> {
    pub(crate) fn from_boxed_cumulative_cents_source(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Height, C>,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let source = LazyPerBlock::from_boxed_height_source::<Identity<C>>(
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
            cached_starts,
            indexes,
        );
        Self {
            block,
            cumulative,
            sum,
        }
    }
}
