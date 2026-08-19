use brk_error::Result;

use bitview_traversable::Traversable;
use brk_cohort::{AgeRangeId, AmountRange, CohortContext, Filter, UTXOGroups};
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{
    AnyStoredVec, CachedBoxedVec, ColumnId, Database, ReadOnlyClone, ReadableColumnarVec, Rw,
    StorageMode,
};

use crate::metrics::{ColumnarAmount, UTXOColumnarMetric, UTXORows};
use bitview_compute::{LazySpotValuePerBlock, PinnedSpotValuePerBlock};

#[derive(Traversable)]
pub struct SupplyTotal<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroups<LazySpotValuePerBlock>,
    #[traversable(flatten)]
    pub matrices: UTXOColumnarMetric<Sats, M>,
    pub addr_balance: ColumnarAmount<Sats, LazySpotValuePerBlock, M>,
    #[traversable(skip)]
    all: PinnedSpotValuePerBlock,
}

impl SupplyTotal {
    pub fn forced_import(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let matrices = UTXOColumnarMetric::forced_import(db, "supply_sats", version)?;
        let all_name = CohortContext::Utxo.metric_name(&Filter::All, "", "supply");
        let all = PinnedSpotValuePerBlock::from_sats_source(
            &all_name,
            version,
            matrices.age_range_matrix.read_only_clone().sum_columns(
                &format!("{all_name}_sats"),
                version,
                AgeRangeId::ALL.iter().copied(),
            ),
            indexes,
            spot_price,
        );
        let cohorts = UTXOGroups::new(|filter, cohort_name| {
            let name = CohortContext::Utxo.metric_name(&filter, cohort_name, "supply");
            if matches!(filter, Filter::All) {
                all.series.clone()
            } else {
                let source = matrices
                    .additive_source(&filter, &format!("{name}_sats"), version)
                    .expect("total-supply cohort source");
                LazySpotValuePerBlock::from_boxed_sats_source(
                    &name, version, source, indexes, spot_price,
                )
            }
        });
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

        Ok(Self {
            cohorts,
            matrices,
            addr_balance,
            all,
        })
    }

    pub fn min_len(&self) -> usize {
        self.matrices.min_len().min(self.addr_balance.len())
    }

    pub fn get(&self, filter: &Filter) -> Option<&LazySpotValuePerBlock> {
        self.cohorts.get(filter)
    }

    pub fn all_supply(&self) -> &CachedBoxedVec<Height, Sats> {
        &self.all.sats
    }

    pub fn all_market_cap(&self) -> &CachedBoxedVec<Height, Cents> {
        &self.all.cents
    }

    #[inline(always)]
    pub fn push(&mut self, rows: UTXORows<Sats>) {
        self.matrices.push(rows);
    }

    #[inline(always)]
    pub fn push_addr_balance(&mut self, row: AmountRange<Sats>) {
        self.addr_balance.push(row);
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.matrices.collect_vecs_mut();
        vecs.push(self.addr_balance.stored_mut());
        vecs
    }
}
