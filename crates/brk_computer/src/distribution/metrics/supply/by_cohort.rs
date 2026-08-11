use brk_cohort::{CohortContext, Filter, UTXOGroupsWithoutAmount};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, CachedBoxedVec, Database, Rw, StorageMode};

use crate::{
    distribution::metrics::{UTXOColumnarMetricWithoutAmount, UTXORows},
    indexes,
    internal::LazySpotValuePerBlock,
};

#[derive(Traversable)]
pub struct SupplyByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroupsWithoutAmount<LazySpotValuePerBlock>,
    #[traversable(flatten)]
    pub matrices: UTXOColumnarMetricWithoutAmount<Sats, M>,
}

impl SupplyByCohort {
    pub(crate) fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        indexes: &indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let matrices =
            UTXOColumnarMetricWithoutAmount::forced_import(db, &format!("{metric}_sats"), version)?;
        let cohorts = UTXOGroupsWithoutAmount::new(|filter, cohort_name| {
            let name = CohortContext::Utxo.metric_name(&filter, cohort_name, metric);
            let source = matrices
                .additive_source(&filter, &format!("{name}_sats"), version)
                .expect("supported supply cohort");
            LazySpotValuePerBlock::from_boxed_sats_source(
                &name, version, source, indexes, spot_price,
            )
        });

        Ok(Self { cohorts, matrices })
    }

    pub(crate) fn get(&self, filter: &Filter) -> Option<&LazySpotValuePerBlock> {
        self.cohorts.get(filter)
    }

    pub(crate) fn min_len(&self) -> usize {
        self.matrices.min_len()
    }

    #[inline(always)]
    pub(crate) fn push(&mut self, rows: UTXORows<Sats>) {
        self.matrices.push(rows);
    }

    pub(crate) fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.matrices.collect_vecs_mut()
    }
}
