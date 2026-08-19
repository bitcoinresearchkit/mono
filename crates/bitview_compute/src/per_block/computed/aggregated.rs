use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::Height;
use schemars::JsonSchema;
use vecdb::{
    Database, Exit, ReadableCloneableVec, ReadableVec, Rw, StorageMode, TypedVec, Version,
};

use crate::{
    CachedWindowStartVec, Identity, LazyPerBlock, LazyPreviousDeltaVec, NumericValue,
    RollingComplete, WindowStarts, Windows,
};

#[derive(Traversable)]
pub struct PerBlockAggregated<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema,
{
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub sum: LazyPreviousDeltaVec<Height, T>,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: LazyPerBlock<T>,
    pub rolling: RollingComplete<T, M>,
}

impl<T> PerBlockAggregated<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn forced_import<V>(
        db: &Database,
        name: &str,
        version: Version,
        cumulative_source: V,
        indexes: &crate::IndexSources,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self>
    where
        V: TypedVec<I = Height, T = T> + ReadableVec<Height, T> + Clone + 'static,
    {
        let sum = LazyPreviousDeltaVec::new(
            &format!("{name}_sum"),
            version,
            cumulative_source.read_only_boxed_clone(),
        );
        let cumulative = LazyPerBlock::from_height_source::<Identity<T>>(
            &format!("{name}_cumulative"),
            version,
            cumulative_source,
            indexes,
        );
        let rolling = RollingComplete::forced_import(
            db,
            name,
            version,
            indexes,
            &cumulative.height,
            cached_starts,
        )?;

        Ok(Self {
            sum,
            cumulative,
            rolling,
        })
    }

    pub fn compute_rest(
        &mut self,
        max_from: Height,
        windows: &WindowStarts<'_>,
        exit: &Exit,
    ) -> Result<()>
    where
        T: From<f64> + Default + Copy + Ord,
        f64: From<T>,
    {
        self.rolling.compute(max_from, windows, &self.sum, exit)?;
        Ok(())
    }
}
