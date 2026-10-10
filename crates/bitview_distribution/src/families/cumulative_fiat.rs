use std::ops::AddAssign;

use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{
    CumulativeSource, FiatType, LazyFiatPerBlockCumulativeWithSums, LazyWindowStartVec,
};
use brk_error::Result;
use brk_types::{Cents, Version};
use vecdb::{AnyStoredVec, Database, PcoVecValue, Rw, StorageMode};

/// One cohort's fiat flow, stored as its running total `{name}_cumulative_cents`.
#[derive(Traversable)]
pub struct CumulativeFiat<C: FiatType + PcoVecValue = Cents, M: StorageMode = Rw> {
    #[traversable(flatten)]
    value: LazyFiatPerBlockCumulativeWithSums<C>,
    #[traversable(hidden)]
    pub stored: CumulativeSource<C, M>,
}

impl<C: FiatType + PcoVecValue + AddAssign + Default> CumulativeFiat<C> {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored = CumulativeSource::import(
            db,
            &format!("{name}_cumulative_cents"),
            version + Version::TWO,
        )?;
        let value = LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
            name,
            version,
            stored.cumulative_source(),
            mappings,
            window_starts,
        );
        Ok(Self { value, stored })
    }

    #[inline(always)]
    pub fn push_block(&mut self, value: C) {
        self.stored.push_block(value);
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.stored.stored_mut()
    }
}
