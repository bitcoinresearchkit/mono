use crate::addr_groups::SizeAndAddrGroups;
use crate::groups::SizeGroups;
use crate::sources::SizeSources;
use bitview_cohort::{AmountRange, CohortContext};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockWithDeltas, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{PartsPerMillionSigned64, StoredI64, StoredU64, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::AmountSources;

#[derive(Traversable)]
pub struct UnspentOutputCount<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: SizeAndAddrGroups<
        LazyPerBlockWithDeltas<StoredU64, StoredI64, PartsPerMillionSigned64>,
        AmountSources<
            StoredU64,
            LazyPerBlockWithDeltas<StoredU64, StoredI64, PartsPerMillionSigned64>,
            M,
        >,
    >,
    #[traversable(hidden)]
    pub stored: SizeSources<StoredU64, M>,
}

impl UnspentOutputCount {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored = SizeSources::forced_import(db, "utxo_count", version)?;
        let cohorts = SizeGroups::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "utxo_count");
            LazyPerBlockWithDeltas::from_height_source(
                &name,
                version,
                stored.get(cohort_id).expect("unspent-output cohort source"),
                Version::TWO,
                mappings,
                window_starts,
            )
        });
        let addr_balance = AmountSources::forced_import(
            db,
            "addrs_utxo_count_by_balance_range",
            CohortContext::Addr,
            "utxo_count",
            version + Version::ONE,
            |name, source| {
                LazyPerBlockWithDeltas::from_height_source(
                    name,
                    version + Version::ONE,
                    source,
                    Version::TWO,
                    mappings,
                    window_starts,
                )
            },
        )?;
        Ok(Self {
            cohorts: SizeAndAddrGroups {
                utxo: cohorts,
                addr_balance,
            },
            stored,
        })
    }

    #[inline(always)]
    pub fn push_addr_balance(&mut self, values: AmountRange<StoredU64>) {
        self.cohorts.addr_balance.push(values);
    }
}
