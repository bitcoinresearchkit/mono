//! Stored cumulative source and rolling statistics for externally supplied block values.

use bitview_collections::Windows;
use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{
    Database, ReadOnlyClone, ReadableCloneableVec, ReadableVec, Rw, StorageMode, VecValue,
};

use crate::{IndexSources, PerBlock, RollingComplete, WindowStarts};

#[derive(Traversable)]
pub struct PerBlockRolling<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema + Quantity,
{
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub(crate) cumulative: PerBlock<T::Sum, M>,
    #[traversable(flatten)]
    pub(crate) rolling: RollingComplete<T, M>,
}

impl<T> PerBlockRolling<T>
where
    T: NumericValue + JsonSchema + Quantity,
{
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Result<Self> {
        let cumulative = PerBlock::import(db, &format!("{name}_cumulative"), version, indexes)?;
        let cumulative_source = cumulative.height.read_only_clone();
        let rolling = RollingComplete::import(
            db,
            name,
            version,
            indexes,
            &cumulative_source,
            window_starts,
        )?;

        Ok(Self {
            cumulative,
            rolling,
        })
    }

    pub fn cumulative_source(&self) -> &(impl ReadableCloneableVec<Height, T::Sum> + use<T>) {
        self.cumulative.resolutions.height_source()
    }

    /// Computes from per-block values of `S`: the distribution in `T`, the cumulative widened into `T::Sum`.
    pub fn compute<S>(
        &mut self,
        max_from: Height,
        windows: &WindowStarts<'_>,
        height_source: &impl ReadableVec<Height, S>,
        exit: &Exit,
    ) -> Result<()>
    where
        T: From<f64> + Default + Copy + Ord,
        S: VecValue + Copy + Into<T::Sum>,
        f64: From<S>,
    {
        self.cumulative
            .height
            .compute_cumulative(max_from, height_source, exit)?;
        self.rolling
            .compute(max_from, windows, height_source, exit)?;
        Ok(())
    }
}
