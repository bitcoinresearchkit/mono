use brk_error::Result;

use bitview_traversable::Traversable;
use brk_cohort::{CohortContext, UTXOGroups};
use brk_types::{StoredU64, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::CumulativeUTXOColumnarMetric;
use bitview_compute::{CachedWindowStartVec, LazyPerBlockCumulativeRolling, Windows};

#[derive(Traversable)]
pub struct SpentOutputCount<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroups<LazyPerBlockCumulativeRolling<StoredU64>>,
    #[traversable(flatten)]
    pub cumulative: CumulativeUTXOColumnarMetric<StoredU64, M>,
}

impl SpentOutputCount {
    pub fn forced_import(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let cumulative = CumulativeUTXOColumnarMetric::forced_import(
            db,
            "spent_utxo_count_cumulative",
            version,
        )?;
        let cohorts = UTXOGroups::new(|filter, cohort_name| {
            let name = CohortContext::Utxo.metric_name(&filter, cohort_name, "spent_utxo_count");
            LazyPerBlockCumulativeRolling::from_boxed_cumulative_source(
                &name,
                version,
                cumulative
                    .matrices
                    .additive_source(&filter, &format!("{name}_cumulative"), version)
                    .expect("spent-output cohort source"),
                cached_starts,
                indexes,
            )
        });
        Ok(Self {
            cohorts,
            cumulative,
        })
    }
}
