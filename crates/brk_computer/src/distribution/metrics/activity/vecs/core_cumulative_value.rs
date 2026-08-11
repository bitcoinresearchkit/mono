use brk_cohort::UTXOGroupsWithoutAmountOrType;
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, Sats, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::{
    distribution::metrics::{
        CumulativeUTXOValueColumnarMetricWithoutAmountOrType, UTXORows, utxo_metric_name,
    },
    indexes,
    internal::{CachedWindowStartVec, LazyValuePerBlockCumulativeRolling, Windows},
};

#[derive(Traversable)]
pub struct CoreCumulativeValueByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroupsWithoutAmountOrType<LazyValuePerBlockCumulativeRolling>,
    pub cumulative: CumulativeUTXOValueColumnarMetricWithoutAmountOrType<M>,
}

impl CoreCumulativeValueByCohort {
    pub(super) fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let cumulative = CumulativeUTXOValueColumnarMetricWithoutAmountOrType::forced_import(
            db,
            &format!("{metric}_cumulative"),
            version,
        )?;
        let cohorts = UTXOGroupsWithoutAmountOrType::new(|filter, cohort_name| {
            let name = utxo_metric_name(&filter, cohort_name, metric);
            let (sats, cents) = cumulative
                .sources(&filter, &name, version)
                .expect("supported core cumulative value cohort");
            LazyValuePerBlockCumulativeRolling::from_boxed_cumulative_sources(
                &name,
                version,
                sats,
                cents,
                indexes,
                cached_starts,
            )
        });
        Ok(Self {
            cohorts,
            cumulative,
        })
    }

    #[inline(always)]
    pub(super) fn push_block(&mut self, sats: UTXORows<Sats>, cents: UTXORows<Cents>) {
        self.cumulative.push_block(sats, cents);
    }

    pub(super) fn min_len(&self) -> usize {
        self.cumulative.min_len()
    }

    pub(super) fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.cumulative.collect_vecs_mut()
    }
}
