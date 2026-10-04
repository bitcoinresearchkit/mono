use bitview_collections::Windows;
use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{Database, ReadableCloneableVec, Rw, StorageMode};

use crate::{CachedSeries, IndexSources, LazyRollingAvgsFromHeight, import_cached};

/// Stored-block fallback for values whose cumulative delta is not exact, such as floats.
#[derive(Traversable)]
pub struct PerBlockRollingAverage<T, C = T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema,
    C: NumericValue + JsonSchema + Quantity,
{
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub block: CachedSeries<Height, T, M>,
    #[traversable(hidden)]
    cumulative: CachedSeries<Height, C, M>,
    #[traversable(flatten)]
    average: LazyRollingAvgsFromHeight<C>,
}

impl<T, C> PerBlockRollingAverage<T, C>
where
    T: NumericValue + JsonSchema + Into<C>,
    C: NumericValue + JsonSchema + Quantity,
{
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Result<Self> {
        let block = import_cached(db, name, version)?;
        let cumulative =
            import_cached(db, &format!("{name}_cumulative"), version + Version::new(3))?;
        let average = LazyRollingAvgsFromHeight::new(
            &format!("{name}_average"),
            version + Version::new(3),
            &cumulative.read_only_boxed_clone(),
            window_starts,
            indexes,
        );

        Ok(Self {
            block,
            cumulative,
            average,
        })
    }

    /// Compute cumulative from already-populated height data. Rolling averages are lazy.
    pub fn compute_rest(&mut self, max_from: Height, exit: &Exit) -> Result<()> {
        self.cumulative
            .compute_cumulative(max_from, &self.block, exit)?;
        Ok(())
    }
}
