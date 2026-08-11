use brk_cohort::{AmountRange, CohortContext, Filter, UTXOGroups};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, CachedBoxedVec, Database, Rw, StorageMode};

use crate::{
    distribution::metrics::{ColumnarAmount, UTXOColumnarMetric, UTXORows, utxo_metric_name},
    indexes,
    internal::LazySpotValuePerBlock,
};

use super::AllSupplyCache;

#[derive(Traversable)]
pub struct SupplyTotal<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroups<LazySpotValuePerBlock>,
    #[traversable(flatten)]
    pub matrices: UTXOColumnarMetric<Sats, M>,
    pub addr_balance: ColumnarAmount<Sats, LazySpotValuePerBlock, M>,
}

impl SupplyTotal {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<(Self, AllSupplyCache)> {
        let matrices = UTXOColumnarMetric::forced_import(db, "supply_sats", version)?;
        let cohorts = UTXOGroups::new(|filter, cohort_name| {
            let name = utxo_metric_name(&filter, cohort_name, "supply");
            LazySpotValuePerBlock::from_boxed_sats_source(
                &name,
                version,
                matrices
                    .additive_source(&filter, &format!("{name}_sats"), version)
                    .expect("total-supply cohort source"),
                indexes,
                spot_price,
            )
        });
        let all_supply = AllSupplyCache::new(cohorts.all.sats.height.clone());
        let addr_balance = ColumnarAmount::forced_import(
            db,
            "addrs_supply_sats_by_balance_range",
            CohortContext::Addr,
            "supply",
            version + Version::ONE,
            |name, source| {
                LazySpotValuePerBlock::from_boxed_sats_source(
                    name,
                    version + Version::ONE,
                    source,
                    indexes,
                    spot_price,
                )
            },
        )?;

        Ok((
            Self {
                cohorts,
                matrices,
                addr_balance,
            },
            all_supply,
        ))
    }

    pub(crate) fn min_len(&self) -> usize {
        self.matrices.min_len().min(self.addr_balance.len())
    }

    pub(crate) fn get(&self, filter: &Filter) -> Option<&LazySpotValuePerBlock> {
        self.cohorts.get(filter)
    }

    #[inline(always)]
    pub(crate) fn push(&mut self, rows: UTXORows<Sats>) {
        self.matrices.push(rows);
    }

    #[inline(always)]
    pub(crate) fn push_addr_balance(&mut self, row: AmountRange<Sats>) {
        self.addr_balance.push(row);
    }

    pub(crate) fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.matrices.collect_vecs_mut();
        vecs.push(self.addr_balance.stored_mut());
        vecs
    }
}
