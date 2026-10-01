//! Stored cumulative and rolling statistics with a lazy one-source block view.

use bitview_collections::Windows;
use bitview_compute::NumericValue;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{Database, LazyVec, ReadableCloneableVec, Rw, StorageMode, VecValue};

use crate::{IndexSources, PerBlock, RollingComplete, WindowStarts};

#[derive(Traversable)]
pub struct PerBlockFull<T, S, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema,
    S: VecValue,
{
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    block: LazyVec<Height, T, Height, S>,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: PerBlock<T, M>,
    #[traversable(flatten)]
    pub rolling: RollingComplete<T, M>,
}

impl<T, S> PerBlockFull<T, S>
where
    T: NumericValue + JsonSchema,
    S: VecValue,
{
    #[allow(clippy::too_many_arguments)]
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, S>,
        compute_block: fn(Height, S) -> T,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Result<Self> {
        let block = LazyVec::init(name, version, source.read_only_boxed_clone(), compute_block);
        let cumulative =
            PerBlock::forced_import(db, &format!("{name}_cumulative"), version, indexes)?;
        let rolling = RollingComplete::forced_import(
            db,
            name,
            version,
            indexes,
            cumulative.resolutions.height_source(),
            window_starts,
        )?;

        Ok(Self {
            block,
            cumulative,
            rolling,
        })
    }

    pub fn compute(
        &mut self,
        max_from: Height,
        windows: &WindowStarts<'_>,
        exit: &Exit,
    ) -> Result<()>
    where
        T: From<f64> + Default + Copy + Ord,
        f64: From<T>,
    {
        self.cumulative
            .height
            .compute_cumulative(max_from, &self.block, exit)?;
        self.rolling.compute(max_from, windows, &self.block, exit)
    }
}
