use bitview_collections::Windows;
use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Height;
use schemars::JsonSchema;
use vecdb::{Database, Ident, ReadableCloneableVec, Rw, StorageMode, Version};

use crate::{IndexSources, LazyPerBlock, LazyPreviousDeltaVec, RollingComplete, WindowStarts};

#[derive(Traversable)]
pub struct PerBlockAggregated<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema + Quantity,
{
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub sum: LazyPreviousDeltaVec<Height, T>,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    cumulative: LazyPerBlock<T>,
    pub rolling: RollingComplete<T, M>,
}

impl<T> PerBlockAggregated<T>
where
    T: NumericValue + JsonSchema + Quantity,
{
    pub fn import<V>(
        db: &Database,
        name: &str,
        version: Version,
        cumulative_source: &V,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Result<Self>
    where
        V: ReadableCloneableVec<Height, T> + ?Sized,
    {
        let sum = LazyPreviousDeltaVec::new(&format!("{name}_sum"), version, cumulative_source);
        let cumulative = LazyPerBlock::from_height_source::<Ident>(
            &format!("{name}_cumulative"),
            version,
            cumulative_source,
            indexes,
        );
        let rolling = RollingComplete::import(
            db,
            name,
            version,
            indexes,
            &cumulative.height,
            window_starts,
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
        self.rolling.compute(max_from, windows, &self.sum, exit)
    }
}
