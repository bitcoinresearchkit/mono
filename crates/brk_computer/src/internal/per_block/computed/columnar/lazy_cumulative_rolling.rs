use brk_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{ColumnId, PcoVec, ReadOnlyColumnarVec};

use super::{super::lazy_cumulative_rolling::lazy_parts, LazyColumnPerBlock};
use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, LazyPreviousDeltaVec, LazyRollingAvgsFromHeight,
        LazyRollingSumsFromHeight, NumericValue, Windows,
    },
};

#[derive(Clone, Traversable)]
pub struct LazyColumnPerBlockCumulativeRolling<T, C>
where
    T: NumericValue + JsonSchema,
    C: ColumnId,
{
    pub block: LazyPreviousDeltaVec<Height, T>,
    pub cumulative: LazyColumnPerBlock<T, C>,
    pub sum: LazyRollingSumsFromHeight<T>,
    pub average: LazyRollingAvgsFromHeight<T>,
}

impl<T, C> LazyColumnPerBlockCumulativeRolling<T, C>
where
    T: NumericValue + JsonSchema,
    C: ColumnId,
{
    pub(crate) fn new(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, T>, C>,
        column: C,
        indexes: &indexes::Vecs,
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
