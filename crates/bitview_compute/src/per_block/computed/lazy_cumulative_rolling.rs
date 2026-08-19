//! Lazy counterpart to `PerBlockCumulativeRolling`.

use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{ColumnId, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, TypedVec};

use crate::{
    CachedWindowStartVec, Identity, LazyColumnPerBlock, LazyPerBlock, LazyPreviousDeltaVec,
    LazyRollingAvgsFromHeight, LazyRollingSumsFromHeight, NumericValue, Windows,
};

pub fn lazy_parts<T>(
    name: &str,
    version: Version,
    cumulative: &(impl ReadableCloneableVec<Height, T> + 'static),
    cached_starts: &Windows<&CachedWindowStartVec>,
    indexes: &crate::IndexSources,
) -> (
    LazyPreviousDeltaVec<Height, T>,
    LazyRollingSumsFromHeight<T>,
    LazyRollingAvgsFromHeight<T>,
)
where
    T: NumericValue + JsonSchema,
{
    (
        LazyPreviousDeltaVec::new(name, version, cumulative.read_only_boxed_clone()),
        LazyRollingSumsFromHeight::new(
            &format!("{name}_sum"),
            version,
            cumulative,
            cached_starts,
            indexes,
        ),
        LazyRollingAvgsFromHeight::new(
            &format!("{name}_average"),
            version,
            cumulative,
            cached_starts,
            indexes,
        ),
    )
}

#[derive(Clone, Traversable)]
pub struct LazyPerBlockCumulativeRolling<T>
where
    T: NumericValue + JsonSchema,
{
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub block: LazyPreviousDeltaVec<Height, T>,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: LazyPerBlock<T>,
    pub sum: LazyRollingSumsFromHeight<T>,
    pub average: LazyRollingAvgsFromHeight<T>,
}

impl<T> LazyPerBlockCumulativeRolling<T>
where
    T: NumericValue + JsonSchema,
{
    fn from_cumulative(
        name: &str,
        version: Version,
        cumulative: LazyPerBlock<T>,
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &crate::IndexSources,
    ) -> Self {
        let (block, sum, average) =
            lazy_parts(name, version, &cumulative.height, cached_starts, indexes);

        Self {
            block,
            cumulative,
            sum,
            average,
        }
    }

    pub fn from_cumulative_source<V>(
        name: &str,
        version: Version,
        source: V,
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &crate::IndexSources,
    ) -> Self
    where
        V: TypedVec<I = Height, T = T> + ReadableVec<Height, T> + Clone + 'static,
    {
        let cumulative = LazyPerBlock::from_height_source::<Identity<T>>(
            &format!("{name}_cumulative"),
            version,
            source,
            indexes,
        );

        Self::from_cumulative(name, version, cumulative, cached_starts, indexes)
    }

    pub fn from_boxed_cumulative_source(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Height, T>,
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &crate::IndexSources,
    ) -> Self {
        let cumulative = LazyPerBlock::from_boxed_height_source::<Identity<T>>(
            &format!("{name}_cumulative"),
            version,
            source,
            indexes,
        );

        Self::from_cumulative(name, version, cumulative, cached_starts, indexes)
    }

    pub fn from_lazy_source(
        name: &str,
        version: Version,
        source: &LazyPerBlock<T>,
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &crate::IndexSources,
    ) -> Self {
        let cumulative = LazyPerBlock::from_lazy::<Identity<T>, T>(
            &format!("{name}_cumulative"),
            version,
            source,
        );

        Self::from_cumulative(name, version, cumulative, cached_starts, indexes)
    }

    pub fn from_column_source<C: ColumnId>(
        name: &str,
        version: Version,
        source: &LazyColumnPerBlock<T, C>,
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &crate::IndexSources,
    ) -> Self {
        let cumulative = LazyPerBlock::from_resolutions::<Identity<T>>(
            &format!("{name}_cumulative"),
            version,
            source.height.read_only_boxed_clone(),
            &source.resolutions,
        );

        Self::from_cumulative(name, version, cumulative, cached_starts, indexes)
    }
}
