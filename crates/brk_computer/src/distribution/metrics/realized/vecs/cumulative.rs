use brk_cohort::{CohortContext, UTXOGroups};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::{
    distribution::metrics::CumulativeUTXOColumnarMetric,
    indexes,
    internal::{CachedWindowStartVec, LazyFiatPerBlockCumulativeWithSums, Windows},
};

#[derive(Traversable)]
pub struct CumulativeRealizedByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroups<LazyFiatPerBlockCumulativeWithSums<Cents>>,
    #[traversable(flatten)]
    pub cumulative: CumulativeUTXOColumnarMetric<Cents, M>,
}

impl CumulativeRealizedByCohort {
    pub(super) fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let cumulative = CumulativeUTXOColumnarMetric::forced_import(
            db,
            &format!("{metric}_cumulative_cents"),
            version,
        )?;
        let cohorts = UTXOGroups::new(|filter, cohort_name| {
            let name = CohortContext::Utxo.metric_name(&filter, cohort_name, metric);
            let source = cumulative
                .matrices
                .additive_source(&filter, &format!("{name}_cumulative_cents"), version)
                .expect("supported cumulative realized cohort");
            LazyFiatPerBlockCumulativeWithSums::from_boxed_cumulative_cents_source(
                &name,
                version,
                source,
                indexes,
                cached_starts,
            )
        });
        Ok(Self {
            cohorts,
            cumulative,
        })
    }
}
