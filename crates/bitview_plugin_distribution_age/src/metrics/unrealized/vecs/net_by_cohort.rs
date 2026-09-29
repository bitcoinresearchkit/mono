use bitview_cohort::{CohortContext, UTXOGroupsWithoutAmountOrType};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazyFiatPerBlock;
use brk_error::Result;
use brk_types::{CentsSigned, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::UTXOCoreSources;

#[derive(Traversable)]
pub struct NetUnrealizedByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroupsWithoutAmountOrType<LazyFiatPerBlock<CentsSigned>>,
    #[traversable(hidden)]
    pub stored: UTXOCoreSources<CentsSigned, M>,
}

impl NetUnrealizedByCohort {
    pub fn forced_import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let metric = "net_unrealized_pnl";
        let stored = UTXOCoreSources::forced_import(db, "net_unrealized_pnl_cents", version)?;
        let cohorts = UTXOGroupsWithoutAmountOrType::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, metric);
            let source = stored
                .get(cohort_id)
                .expect("supported net unrealized cohort");
            LazyFiatPerBlock::from_cents_source(&name, version, source, mappings)
        });
        Ok(Self { cohorts, stored })
    }
}
