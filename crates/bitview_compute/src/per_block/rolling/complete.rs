//! RollingComplete - Lazy rolling sums + stored rolling distribution per window.

use brk_error::Result;

use bitview_traversable::Traversable;
use brk_exit::Exit;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode};

use crate::{
    CachedWindowStartVec, LazyRollingAvgsFromHeight, LazyRollingSumsFromHeight, NumericValue,
    RollingDistribution, WindowStarts, Windows,
};

/// Lazy rolling sums + lazy rolling averages + stored rolling distribution (7 stats × 4 windows).
#[derive(Traversable)]
pub struct RollingComplete<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema,
{
    pub sum: LazyRollingSumsFromHeight<T>,
    pub average: LazyRollingAvgsFromHeight<T>,
    #[traversable(flatten)]
    pub distribution: RollingDistribution<T, M>,
}

impl<T> RollingComplete<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
        cumulative: &(impl ReadableCloneableVec<Height, T> + 'static),
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let sum = LazyRollingSumsFromHeight::new(
            &format!("{name}_sum"),
            version,
            cumulative,
            cached_starts,
            indexes,
        );
        let average = LazyRollingAvgsFromHeight::new(
            &format!("{name}_average"),
            version,
            cumulative,
            cached_starts,
            indexes,
        );
        let distribution = RollingDistribution::forced_import(db, name, version, indexes)?;

        Ok(Self {
            sum,
            average,
            distribution,
        })
    }

    /// Compute rolling distribution stats across all 4 windows.
    pub fn compute(
        &mut self,
        max_from: Height,
        windows: &WindowStarts<'_>,
        source: &impl ReadableVec<Height, T>,
        exit: &Exit,
    ) -> Result<()>
    where
        T: From<f64> + Default + Copy + Ord,
        f64: From<T>,
    {
        self.distribution
            .compute_distribution(max_from, windows, source, exit)
    }
}
