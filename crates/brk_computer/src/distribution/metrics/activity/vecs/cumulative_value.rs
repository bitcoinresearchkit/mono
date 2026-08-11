use brk_cohort::{AmountRange, CohortContext, UTXOGroups};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, Sats, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::{
    distribution::metrics::{
        ColumnarAmountValue, CumulativeUTXOValueColumnarMetric, UTXORows, utxo_metric_name,
    },
    indexes,
    internal::{CachedWindowStartVec, LazyValuePerBlockCumulativeRolling, Windows},
};

#[derive(Traversable)]
pub struct CumulativeValueByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroups<LazyValuePerBlockCumulativeRolling>,
    pub cumulative: CumulativeUTXOValueColumnarMetric<M>,
    pub addr_balance: ColumnarAmountValue<LazyValuePerBlockCumulativeRolling, M>,
}

impl CumulativeValueByCohort {
    pub(super) fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let cumulative = CumulativeUTXOValueColumnarMetric::forced_import(
            db,
            &format!("{metric}_cumulative"),
            version,
        )?;
        let cohorts = UTXOGroups::new(|filter, cohort_name| {
            let name = utxo_metric_name(&filter, cohort_name, metric);
            let (sats, cents) = cumulative
                .sources(&filter, &name, version)
                .expect("supported cumulative value cohort");
            LazyValuePerBlockCumulativeRolling::from_boxed_cumulative_sources(
                &name,
                version,
                sats,
                cents,
                indexes,
                cached_starts,
            )
        });
        let addr_version = version + Version::ONE;
        let addr_balance = ColumnarAmountValue::forced_import(
            db,
            &format!("addrs_{metric}_cumulative_by_balance_range"),
            CohortContext::Addr,
            metric,
            addr_version,
            |name, sats, cents| {
                LazyValuePerBlockCumulativeRolling::from_boxed_cumulative_sources(
                    name,
                    addr_version,
                    sats,
                    cents,
                    indexes,
                    cached_starts,
                )
            },
        )?;
        Ok(Self {
            cohorts,
            cumulative,
            addr_balance,
        })
    }

    #[inline(always)]
    pub(super) fn push_block(&mut self, sats: UTXORows<Sats>, cents: UTXORows<Cents>) {
        self.cumulative.push_block(sats, cents);
    }

    #[inline(always)]
    pub(super) fn push_addr_balance(
        &mut self,
        sats: &AmountRange<Sats>,
        cents: &AmountRange<Cents>,
    ) {
        self.addr_balance.push_cumulative(sats, cents);
    }

    pub(super) fn min_len(&self) -> usize {
        self.cumulative.min_len().min(self.addr_balance.len())
    }

    pub(super) fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.cumulative.collect_vecs_mut();
        vecs.extend(self.addr_balance.collect_vecs_mut());
        vecs
    }
}
