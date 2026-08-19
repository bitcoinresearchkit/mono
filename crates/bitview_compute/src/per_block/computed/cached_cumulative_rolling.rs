//! Stored cumulative source with pinned cached reads and lazy derived views.

use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{
    AnyStoredVec, AnyVec, CachedBoxedVec, Database, ReadOnlyClone, ReadableVec, Rw, StorageMode,
    WritableVec,
};

use super::lazy_cumulative_rolling::lazy_parts;
use crate::{
    CachedPerBlock, CachedWindowStartVec, LazyPreviousDeltaVec, LazyRollingAvgsFromHeight,
    LazyRollingSumsFromHeight, NumericValue, Windows,
};

/// Like [`super::PerBlockCumulativeRolling`], with a pinned cache owned by its
/// cumulative height source.
#[derive(Traversable)]
pub struct CachedPerBlockCumulativeRolling<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema,
{
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub block: LazyPreviousDeltaVec<Height, T>,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: CachedPerBlock<T, M>,
    pub sum: LazyRollingSumsFromHeight<T>,
    pub average: LazyRollingAvgsFromHeight<T>,
    #[traversable(skip)]
    last_cumulative: Option<(usize, T)>,
}

impl<T> CachedPerBlockCumulativeRolling<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let cumulative =
            CachedPerBlock::forced_import(db, &format!("{name}_cumulative"), version, indexes)?;
        let cumulative_source = cumulative.height.read_only_clone();
        let (block, sum, average) =
            lazy_parts(name, version, &cumulative_source, cached_starts, indexes);
        let last_cumulative = cumulative
            .height
            .collect_last()
            .map(|value| (cumulative.height.len(), value));

        Ok(Self {
            block,
            cumulative,
            sum,
            average,
            last_cumulative,
        })
    }

    pub fn cached_cumulative(&self) -> CachedBoxedVec<Height, T> {
        self.cumulative.height.read_only_cached_boxed_clone()
    }

    #[inline(always)]
    pub fn push_block(&mut self, value: T)
    where
        T: Copy,
    {
        let len = self.cumulative.height.len();
        let mut cumulative = match self.last_cumulative {
            Some((cached_len, value)) if cached_len == len => value,
            _ => self.cumulative.height.collect_last().unwrap_or_default(),
        };
        cumulative += value;
        self.cumulative.height.inner.push(cumulative);
        self.last_cumulative = Some((len + 1, cumulative));
    }

    pub fn validate_and_truncate(&mut self, version: Version, height: Height) -> Result<()> {
        self.cumulative
            .height
            .inner
            .validate_and_truncate(version, height)?;
        Ok(())
    }

    pub fn truncate_if_needed_at(&mut self, len: usize) -> Result<()> {
        self.cumulative.height.inner.truncate_if_needed_at(len)?;
        Ok(())
    }

    pub fn write(&mut self) -> Result<()> {
        self.cumulative.height.inner.write()?;
        Ok(())
    }
}
