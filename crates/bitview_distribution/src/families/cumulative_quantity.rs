use bitview_compute::{NumericValue, Quantity};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{
    CumulativeSource, LazyPerBlockCumulativeRolling, LazyWindowStartVec, shapes::Windows,
};
use brk_error::Result;
use brk_types::Version;
use schemars::JsonSchema;
use vecdb::{Database, Rw, StorageMode};

/// One cohort's flow of a quantity, stored as its running total `{name}_cumulative`.
#[derive(Traversable)]
pub struct CumulativeQuantity<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    #[traversable(flatten)]
    pub value: LazyPerBlockCumulativeRolling<T>,
    #[traversable(hidden)]
    pub stored: CumulativeSource<T, M>,
}

impl<T> CumulativeQuantity<T>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    pub(crate) fn import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored =
            CumulativeSource::import(db, &format!("{name}_cumulative"), version + Version::TWO)?;
        let value = LazyPerBlockCumulativeRolling::from_cumulative_source(
            name,
            version,
            stored.cumulative_source(),
            window_starts,
            mappings,
        );
        Ok(Self { value, stored })
    }

    #[inline(always)]
    pub(crate) fn push_block(&mut self, value: T) {
        self.stored.push_block(value);
    }
}
