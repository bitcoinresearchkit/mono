use bitview_cohort::{CohortContext, CohortId};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, FiatType, LazyFiatPerBlock};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{AnyStoredVec, Database, PcoVecValue, Rw, StorageMode, WritableVec};

use super::import_stored;

/// One cohort's fiat amount, stored in cents as `{metric}_cents` and shown in every fiat unit.
#[derive(Traversable)]
pub struct Fiat<C: FiatType + PcoVecValue = Cents, M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub value: LazyFiatPerBlock<C>,
    #[traversable(hidden)]
    pub stored: CachedSeries<Height, C, M>,
}

impl<C: FiatType + PcoVecValue> Fiat<C> {
    pub fn import(
        db: &Database,
        cohort: CohortId,
        metric: &str,
        version: Version,
        mappings: &MappingsVecs,
    ) -> Result<Self> {
        let stored = import_stored(db, cohort, &format!("{metric}_cents"), version)?;
        let value = LazyFiatPerBlock::from_cents_source(
            &CohortContext::Utxo.metric_name(cohort, metric),
            version,
            &stored,
            mappings,
        );
        Ok(Self { value, stored })
    }

    #[inline(always)]
    pub fn push(&mut self, value: C) {
        self.stored.push(value);
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        &mut self.stored
    }
}
