use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{ColumnId, PcoVec, ReadOnlyColumnarVec};

use super::{super::lazy_cumulative_rolling::lazy_parts, LazyColumnPerBlock};
use crate::{
    CachedWindowStartVec, LazyPreviousDeltaVec, LazyRollingAvgsFromHeight,
    LazyRollingSumsFromHeight, NumericValue, Windows,
};

#[derive(Clone, Traversable)]
pub struct LazyColumnPerBlockCumulativeRolling<T, C>
where
    T: NumericValue + JsonSchema,
    C: ColumnId,
{
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub block: LazyPreviousDeltaVec<Height, T>,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: LazyColumnPerBlock<T, C>,
    pub sum: LazyRollingSumsFromHeight<T>,
    pub average: LazyRollingAvgsFromHeight<T>,
}

impl<T, C> LazyColumnPerBlockCumulativeRolling<T, C>
where
    T: NumericValue + JsonSchema,
    C: ColumnId,
{
    pub fn new(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, T>, C>,
        column: C,
        indexes: &crate::IndexSources,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let cumulative = LazyColumnPerBlock::new(
            &format!("{name}_cumulative"),
            version,
            source,
            column,
            indexes,
        );
        let (block, sum, average) =
            lazy_parts(name, version, &cumulative.height, cached_starts, indexes);

        Self {
            block,
            cumulative,
            sum,
            average,
        }
    }
}
