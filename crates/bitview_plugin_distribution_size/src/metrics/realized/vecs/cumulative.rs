use crate::addr_groups::SizeAndAddrGroups;
use crate::cumulative::CumulativeSizeSources;
use crate::groups::SizeGroups;
use bitview_cohort::CohortContext;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFiatPerBlockCumulativeWithSums, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{Cents, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::AmountSources;

#[derive(Traversable)]
pub struct CumulativeRealizedByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    /// Includes spends grouped by the address's pre-spend balance.
    pub cohorts: SizeAndAddrGroups<
        LazyFiatPerBlockCumulativeWithSums<Cents>,
        AmountSources<Cents, LazyFiatPerBlockCumulativeWithSums<Cents>, M>,
    >,
    #[traversable(hidden)]
    pub stored: CumulativeSizeSources<Cents, M>,
}

impl CumulativeRealizedByCohort {
    pub fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored = CumulativeSizeSources::forced_import(
            db,
            &format!("{metric}_cumulative_cents"),
            version,
        )?;
        let cohorts = SizeGroups::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, metric);
            let source = stored
                .stored
                .get(cohort_id)
                .expect("supported stored realized cohort");
            LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
                &name,
                version,
                source,
                mappings,
                window_starts,
            )
        });
        let addr_version = version + Version::ONE;
        let addr_balance = AmountSources::forced_import(
            db,
            &format!("addrs_{metric}_cumulative_cents_by_balance_range"),
            CohortContext::Addr,
            metric,
            addr_version,
            |name, source| {
                LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
                    name,
                    addr_version,
                    source,
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
}
