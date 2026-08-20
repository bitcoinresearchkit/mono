use brk_error::Result;

use bitview_cohort::{CohortContext, UTXOGroupsWithoutAmountOrType};
use bitview_traversable::Traversable;
use brk_types::{StoredF64, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::CumulativeUTXOColumnarMetricWithoutAmountOrType;
use bitview_compute::{CachedWindowStartVec, LazyPerBlockCumulativeRolling, Windows};

#[derive(Traversable)]
pub struct CoindaysDestroyedByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroupsWithoutAmountOrType<LazyPerBlockCumulativeRolling<StoredF64>>,
    #[traversable(flatten)]
    pub cumulative: CumulativeUTXOColumnarMetricWithoutAmountOrType<StoredF64, M>,
}

impl CoindaysDestroyedByCohort {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let cumulative = CumulativeUTXOColumnarMetricWithoutAmountOrType::forced_import(
            db,
            "coindays_destroyed_cumulative",
            version,
        )?;
        let cohorts = UTXOGroupsWithoutAmountOrType::new(|filter, cohort_name| {
            let name = CohortContext::Utxo.metric_name(&filter, cohort_name, "coindays_destroyed");
            let source = cumulative
                .matrices
                .additive_source(&filter, &format!("{name}_cumulative"), version)
                .expect("supported coindays-destroyed cohort");
            LazyPerBlockCumulativeRolling::from_boxed_cumulative_source(
                &name,
                version,
                source,
                cached_starts,
                mappings,
            )
        });
        Ok(Self {
            cohorts,
            cumulative,
        })
    }
}
