use brk_cohort::{AmountRange, CohortContext, UTXOGroups};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{PartsPerMillionSigned64, StoredI64, StoredU64, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::{
    distribution::metrics::{ColumnarAmount, UTXOColumnarMetric},
    indexes,
    internal::{CachedWindowStartVec, LazyPerBlockWithDeltas, Windows},
};

#[derive(Traversable)]
pub struct UnspentOutputCount<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroups<LazyPerBlockWithDeltas<StoredU64, StoredI64, PartsPerMillionSigned64>>,
    #[traversable(flatten)]
    pub matrices: UTXOColumnarMetric<StoredU64, M>,
    pub addr_balance: ColumnarAmount<
        StoredU64,
        LazyPerBlockWithDeltas<StoredU64, StoredI64, PartsPerMillionSigned64>,
        M,
    >,
}

impl UnspentOutputCount {
    pub fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let matrices = UTXOColumnarMetric::forced_import(db, "utxo_count", version)?;
        let cohorts = UTXOGroups::new(|filter, cohort_name| {
            let name = CohortContext::Utxo.metric_name(&filter, cohort_name, "utxo_count");
            LazyPerBlockWithDeltas::from_boxed_height_source(
                &name,
                version,
                matrices
                    .additive_source(&filter, &name, version)
                    .expect("unspent-output cohort source"),
                Version::TWO,
                indexes,
                cached_starts,
            )
        });
        let addr_balance = ColumnarAmount::forced_import(
            db,
            "addrs_utxo_count_by_balance_range",
            CohortContext::Addr,
            "utxo_count",
            version + Version::ONE,
            |name, source| {
                LazyPerBlockWithDeltas::from_boxed_height_source(
                    name,
                    version + Version::ONE,
                    source,
                    Version::TWO,
                    indexes,
                    cached_starts,
                )
            },
        )?;
        Ok(Self {
            cohorts,
            matrices,
            addr_balance,
        })
    }

    #[inline(always)]
    pub fn push_addr_balance(&mut self, row: AmountRange<StoredU64>) {
        self.addr_balance.push(row);
    }
}
