//! RollingComplete - Lazy rolling sums + stored rolling distribution per window.

use crate::RollingTotals;
use bitview_collections::Windows;
use bitview_compute::NumericValue;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode};

use crate::{IndexSources, RollingDistribution, WindowStarts};

/// Lazy rolling sums + lazy rolling averages + stored rolling distribution (7 stats × 4 windows).
#[derive(Deref, DerefMut, Traversable)]
pub struct RollingComplete<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema,
{
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    rolling: RollingTotals<T>,
    #[traversable(flatten)]
    pub(crate) distribution: RollingDistribution<T, M>,
}

impl<T> RollingComplete<T>
where
    T: NumericValue + JsonSchema,
{
    pub(crate) fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        cumulative: &impl ReadableCloneableVec<Height, T>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Result<Self> {
        let rolling = RollingTotals::new(name, version, cumulative, window_starts, indexes);
        let distribution = RollingDistribution::import(db, name, version, indexes)?;

        Ok(Self {
            rolling,
            distribution,
        })
    }

    /// Compute rolling distribution stats across all 4 windows.
    pub(crate) fn compute(
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
