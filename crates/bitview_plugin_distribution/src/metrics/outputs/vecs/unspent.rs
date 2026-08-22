use brk_error::Result;

use bitview_cohort::{AmountRange, CohortContext, UTXOGroups};
use bitview_traversable::Traversable;
use brk_types::{PartsPerMillionSigned64, StoredI64, StoredU64, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::{ColumnarAmount, UTXOColumnarMetric};
use bitview_compute::{CachedWindowStartVec, LazyPerBlockWithDeltas, Windows};

#[derive(Traversable)]
pub struct UnspentOutputCount<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroups<LazyPerBlockWithDeltas<StoredU64, StoredI64, PartsPerMillionSigned64>>,
    #[traversable(flatten)]
    pub matrices: UTXOColumnarMetric<StoredU64, M>,
    /// Groups funded addresses by their balance at the represented block.
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
        mappings: &bitview_plugin_mappings::Vecs,
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
                mappings,
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
                    mappings,
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
