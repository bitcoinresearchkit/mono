use brk_cohort::UTXOGroupsWithoutAmountOrType;
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::{
    distribution::metrics::{CumulativeUTXOColumnarMetricWithoutAmountOrType, utxo_metric_name},
    indexes,
    internal::{CachedWindowStartVec, LazyFiatPerBlockCumulativeWithSums, Windows},
};

#[derive(Traversable)]
pub struct CumulativeValueDestroyedByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroupsWithoutAmountOrType<LazyFiatPerBlockCumulativeWithSums<Cents>>,
    #[traversable(flatten)]
    pub cumulative: CumulativeUTXOColumnarMetricWithoutAmountOrType<Cents, M>,
}

impl CumulativeValueDestroyedByCohort {
    pub(super) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let metric = "value_destroyed";
        let cumulative = CumulativeUTXOColumnarMetricWithoutAmountOrType::forced_import(
            db,
            "value_destroyed_cumulative_cents",
            version,
        )?;
        let cohorts = UTXOGroupsWithoutAmountOrType::new(|filter, cohort_name| {
            let name = utxo_metric_name(&filter, cohort_name, metric);
            let source = cumulative
                .matrices
                .additive_source(&filter, &format!("{name}_cumulative_cents"), version)
                .expect("supported value-destroyed cohort");
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
