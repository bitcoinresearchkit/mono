//! Stored cumulative and rolling statistics with a lazy one-source block view.

use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{
    Database, Exit, LazyVec, ReadableCloneableVec, ReadableVec, Rw, StorageMode, VecValue,
};

use crate::{CachedWindowStartVec, NumericValue, PerBlock, RollingComplete, WindowStarts, Windows};

#[derive(Traversable)]
pub struct PerBlockFull<T, S, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema,
    S: VecValue,
{
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub block: LazyVec<Height, T, Height, S>,
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
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, S> + 'static),
        compute_block: fn(Height, S) -> T,
        indexes: &crate::IndexSources,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let block = LazyVec::init(name, version, source.read_only_boxed_clone(), compute_block);
        let cumulative =
            PerBlock::forced_import(db, &format!("{name}_cumulative"), version, indexes)?;
        let rolling = RollingComplete::forced_import(
            db,
            name,
            version,
            indexes,
            &cumulative.height,
            cached_starts,
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
        compute_rest(
            &mut self.cumulative,
            &mut self.rolling,
            &self.block,
            max_from,
            windows,
            exit,
        )
    }
}

fn compute_rest<T>(
    cumulative: &mut PerBlock<T>,
    rolling: &mut RollingComplete<T>,
    source: &impl ReadableVec<Height, T>,
    max_from: Height,
    windows: &WindowStarts<'_>,
    exit: &Exit,
) -> Result<()>
where
    T: NumericValue + JsonSchema + From<f64> + Default + Copy + Ord,
    f64: From<T>,
{
    cumulative
        .height
        .compute_cumulative(max_from, source, exit)?;
    rolling.compute(max_from, windows, source, exit)?;
    Ok(())
}
