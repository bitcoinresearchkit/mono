use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode, VecValue};

use crate::{IndexSources, RollingDistribution, RollingTotals, WindowStarts, shapes::Windows};

/// Lazy rolling sums and averages with stored rolling distribution statistics. Sums and
/// statistics are in `T::Sum`, averages in its `Quantity::Fract`.
#[derive(Deref, DerefMut, Traversable)]
pub struct RollingComplete<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema + Quantity,
{
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    rolling: RollingTotals<T::Sum>,
    #[traversable(flatten)]
    pub(crate) distribution: RollingDistribution<T::Sum, M>,
}

impl<T> RollingComplete<T>
where
    T: NumericValue + JsonSchema + Quantity,
{
    pub(crate) fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        cumulative: &impl ReadableCloneableVec<Height, T::Sum>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Result<Self> {
        let rolling = RollingTotals::new(name, version, cumulative, window_starts, indexes);
        let distribution = RollingDistribution::import(db, name, version + Version::ONE, indexes)?;

        Ok(Self {
            rolling,
            distribution,
        })
    }

    pub(crate) fn compute<S>(
        &mut self,
        max_from: Height,
        windows: &WindowStarts<'_>,
        source: &impl ReadableVec<Height, S>,
        exit: &Exit,
    ) -> Result<()>
    where
        T::Sum: From<f64> + Default + Copy + Ord,
        S: VecValue + Copy,
        f64: From<S>,
    {
        self.distribution
            .compute_distribution(max_from, windows, source, exit)
    }
}
