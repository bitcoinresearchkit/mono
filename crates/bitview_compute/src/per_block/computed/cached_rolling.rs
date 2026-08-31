//! CachedPerBlockRolling - cached cumulative + lazy views + RollingComplete.
//!
//! For metrics derived from indexer sources (no stored height vec).
//! Cumulative gets its own CachedPerBlock so it has LazyAggVec index views too.

use brk_error::Result;

use bitview_traversable::Traversable;
use brk_exit::Exit;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{CachedBoxedVec, Database, ReadOnlyClone, ReadableVec, Rw, StorageMode};

use crate::{
    CachedPerBlock, CachedWindowStartVec, NumericValue, RollingComplete, WindowStarts, Windows,
};

#[derive(Traversable)]
pub struct CachedPerBlockRolling<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema,
{
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: CachedPerBlock<T, M>,
    #[traversable(flatten)]
    pub rolling: RollingComplete<T, M>,
}

impl<T> CachedPerBlockRolling<T>
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
        let rolling = RollingComplete::forced_import(
            db,
            name,
            version,
            indexes,
            &cumulative_source,
            cached_starts,
        )?;

        Ok(Self {
            cumulative,
            rolling,
        })
    }

    pub fn cached_cumulative(&self) -> CachedBoxedVec<Height, T> {
        self.cumulative.height.read_only_cached_boxed_clone()
    }

    pub fn compute(
        &mut self,
        max_from: Height,
        windows: &WindowStarts<'_>,
        height_source: &impl ReadableVec<Height, T>,
        exit: &Exit,
    ) -> Result<()>
    where
        T: From<f64> + Default + Copy + Ord,
        f64: From<T>,
    {
        self.cumulative
            .height
            .inner
            .compute_cumulative(max_from, height_source, exit)?;
        self.rolling
            .compute(max_from, windows, height_source, exit)?;
        Ok(())
    }
}
