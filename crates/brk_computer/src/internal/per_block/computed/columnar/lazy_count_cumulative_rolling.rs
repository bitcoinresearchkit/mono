use brk_traversable::Traversable;
use brk_types::{Height, StoredU16, StoredU64, Version};
use vecdb::{
    CachedReadableVec, CachedVec, ColumnId, LazyVec, PcoVec, ReadOnlyColumnarVec,
    ReadableCloneableVec, ReadableColumnarVec,
};

use crate::{
    indexes,
    internal::{
        CachedBlockCountReader, CachedWindowStartVec, Identity, LazyPerBlock,
        LazyRollingAvgsFromHeight, LazyRollingSumsFromHeight, StoredU16ToStoredU64, Windows,
    },
};

#[derive(Clone, Traversable)]
pub struct LazyColumnCountPerBlockCumulativeRolling {
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub block: LazyVec<Height, StoredU64, Height, StoredU16>,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: LazyPerBlock<StoredU64>,
    pub sum: LazyRollingSumsFromHeight<StoredU64>,
    pub average: LazyRollingAvgsFromHeight<StoredU64>,
    #[traversable(skip)]
    cached_cumulative: CachedBlockCountReader,
}

impl LazyColumnCountPerBlockCumulativeRolling {
    pub(crate) fn new<C>(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, StoredU16>, C>,
        column: C,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self
    where
        C: ColumnId,
    {
        let column = CachedVec::wrap(source.column(name, version, column));
        let cached_cumulative = CachedBlockCountReader::new(column.cached_boxed_clone());
        let block = LazyVec::transformed::<StoredU16ToStoredU64>(
            name,
            version,
            column.read_only_boxed_clone(),
        );
        let cumulative = LazyPerBlock::from_height_source::<Identity<StoredU64>>(
            &format!("{name}_cumulative"),
            version,
            cached_cumulative.clone(),
            indexes,
        );
        let sum = LazyRollingSumsFromHeight::from_compact_cumulative(
            &format!("{name}_sum"),
            version,
            &cached_cumulative,
            cached_starts,
            indexes,
        );
        let average = LazyRollingAvgsFromHeight::new(
            &format!("{name}_average"),
            version,
            &cached_cumulative,
            cached_starts,
            indexes,
        );

        Self {
            block,
            cumulative,
            sum,
            average,
            cached_cumulative,
        }
    }

    #[inline(always)]
    pub(crate) fn cached_cumulative(&self) -> CachedBlockCountReader {
        self.cached_cumulative.clone()
    }

    pub(crate) fn invalidate(&self) {
        self.cached_cumulative.invalidate();
    }
}
