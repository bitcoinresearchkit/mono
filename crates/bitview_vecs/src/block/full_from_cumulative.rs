use bitview_collections::Windows;
use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{Database, Ident, ReadableCloneableVec, Rw, StorageMode};

use crate::{IndexSources, LazyPerBlock, LazyPreviousDeltaVec, RollingComplete, WindowStarts};

/// Per-block and rolling views backed by one canonical cumulative source.
#[derive(Traversable)]
pub struct PerBlockFullFromCumulative<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema + Quantity,
{
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    block: LazyPreviousDeltaVec<Height, T::Sum>,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: LazyPerBlock<T::Sum>,
    #[traversable(flatten)]
    pub rolling: RollingComplete<T, M>,
}

impl<T> PerBlockFullFromCumulative<T>
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
        V: ReadableCloneableVec<Height, T::Sum> + ?Sized,
    {
        let block = LazyPreviousDeltaVec::new(name, version, cumulative_source);
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
        f64: From<T::Sum>,
    {
        self.rolling.compute(max_from, windows, &self.block, exit)
    }
}
