use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{Database, EagerVec, Exit, ImportableVec, PcoVec, Rw, StorageMode};

use crate::{CachedWindowStartVec, LazyRollingAvgsFromHeight, NumericValue, Windows};

/// Stored-block fallback for values whose cumulative delta is not exact, such as floats.
#[derive(Traversable)]
pub struct PerBlockRollingAverage<T, C = T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema,
    C: NumericValue + JsonSchema,
{
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub block: M::Stored<EagerVec<PcoVec<Height, T>>>,
    #[traversable(hidden)]
    cumulative: M::Stored<EagerVec<PcoVec<Height, C>>>,
    #[traversable(flatten)]
    pub average: LazyRollingAvgsFromHeight<C>,
}

impl<T, C> PerBlockRollingAverage<T, C>
where
    T: NumericValue + JsonSchema + Into<C>,
    C: NumericValue + JsonSchema,
{
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let block: EagerVec<PcoVec<Height, T>> = EagerVec::forced_import(db, name, version)?;
        let cumulative: EagerVec<PcoVec<Height, C>> =
            EagerVec::forced_import(db, &format!("{name}_cumulative"), version + Version::TWO)?;
        let average = LazyRollingAvgsFromHeight::new(
            &format!("{name}_average"),
            version + Version::TWO,
            &cumulative,
            cached_starts,
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
