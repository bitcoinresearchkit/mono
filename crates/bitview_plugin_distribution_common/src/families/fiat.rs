use bitview_cohort::{CohortContext, CohortGroup};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{CohortSources, FiatType, LazyFiatPerBlock};
use brk_error::Result;
use brk_types::{Cents, Version};
use vecdb::{Database, PcoVecValue, Rw, StorageMode};

/// A fiat amount per cohort, stored in cents as `{metric}_cents` and shown in every fiat unit.
#[derive(Traversable)]
pub struct FiatByCohort<G: CohortGroup, C: FiatType + PcoVecValue = Cents, M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: G::Of<LazyFiatPerBlock<C>>,
    #[traversable(hidden)]
    pub stored: CohortSources<G, C, M>,
}

impl<G: CohortGroup, C: FiatType + PcoVecValue> FiatByCohort<G, C> {
    pub fn import(
        db: &Database,
        metric: &str,
        version: Version,
        mappings: &MappingsVecs,
    ) -> Result<Self> {
        let stored = CohortSources::import(db, &format!("{metric}_cents"), version)?;
        let cohorts = G::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, metric);
            let source = stored.get(cohort_id).expect("fiat cohort source");
            LazyFiatPerBlock::from_cents_source(&name, version, source, mappings)
        });
        Ok(Self { cohorts, stored })
    }
}
