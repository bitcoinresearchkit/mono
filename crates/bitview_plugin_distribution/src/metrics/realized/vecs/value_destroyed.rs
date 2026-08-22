use brk_error::Result;

use bitview_cohort::{CohortContext, UTXOGroupsWithoutAmountOrType};
use bitview_traversable::Traversable;
use brk_types::{Cents, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::CumulativeUTXOColumnarMetricWithoutAmountOrType;
use bitview_compute::{CachedWindowStartVec, LazyFiatPerBlockCumulativeRolling, Windows};

#[derive(Traversable)]
pub struct CumulativeValueDestroyedByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroupsWithoutAmountOrType<LazyFiatPerBlockCumulativeRolling<Cents>>,
    #[traversable(flatten)]
    pub cumulative: CumulativeUTXOColumnarMetricWithoutAmountOrType<Cents, M>,
}

impl CumulativeValueDestroyedByCohort {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let metric = "value_destroyed";
        let cumulative = CumulativeUTXOColumnarMetricWithoutAmountOrType::forced_import(
            db,
            "value_destroyed_cumulative_cents",
            version,
        )?;
        let cohorts = UTXOGroupsWithoutAmountOrType::new(|filter, cohort_name| {
            let name = CohortContext::Utxo.metric_name(&filter, cohort_name, metric);
            let source = cumulative
                .matrices
                .additive_source(&filter, &format!("{name}_cumulative_cents"), version)
                .expect("supported value-destroyed cohort");
            LazyFiatPerBlockCumulativeRolling::from_boxed_cumulative_cents_source(
                &name,
                version,
                source,
                mappings,
                cached_starts,
            )
        });
        Ok(Self {
            cohorts,
            cumulative,
        })
    }
}
