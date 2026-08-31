use brk_error::Result;

use bitview_traversable::Traversable;
use brk_exit::Exit;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode, TypedVec};

use crate::{
    CachedWindowStartVec, Identity, LazyPerBlock, LazyPreviousDeltaVec, NumericValue,
    RollingComplete, WindowStarts, Windows,
};

/// Per-block and rolling views backed by one canonical cumulative source.
#[derive(Traversable)]
pub struct PerBlockFullFromCumulative<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema,
{
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub block: LazyPreviousDeltaVec<Height, T>,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: LazyPerBlock<T>,
    #[traversable(flatten)]
    pub rolling: RollingComplete<T, M>,
}

impl<T> PerBlockFullFromCumulative<T>
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
        let block =
            LazyPreviousDeltaVec::new(name, version, cumulative_source.read_only_boxed_clone());
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
        self.rolling.compute(max_from, windows, &self.block, exit)
    }
}
