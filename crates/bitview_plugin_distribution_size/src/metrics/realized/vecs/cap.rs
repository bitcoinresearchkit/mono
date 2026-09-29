use crate::addr_groups::SizeAndAddrGroups;
use crate::groups::SizeGroups;
use crate::sources::SizeSources;
use bitview_cohort::CohortContext;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFiatPerBlockWithDeltas, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{Cents, CentsSigned, PartsPerMillionSigned64, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::AmountSources;

#[derive(Traversable)]
pub struct RealizedCapByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: SizeAndAddrGroups<
        LazyFiatPerBlockWithDeltas<Cents, CentsSigned, PartsPerMillionSigned64>,
        AmountSources<
            Cents,
            LazyFiatPerBlockWithDeltas<Cents, CentsSigned, PartsPerMillionSigned64>,
            M,
        >,
    >,
    #[traversable(hidden)]
    pub stored: SizeSources<Cents, M>,
}

impl RealizedCapByCohort {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored = SizeSources::forced_import(db, "realized_cap_cents", version)?;
        let cohorts = SizeGroups::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "realized_cap");
            LazyFiatPerBlockWithDeltas::from_cents_source(
                &name,
                version,
                stored.get(cohort_id).expect("realized-cap cohort source"),
                Version::TWO,
                mappings,
                window_starts,
            )
        });
        let addr_version = version + Version::ONE;
        let addr_balance = AmountSources::forced_import(
            db,
            "addrs_realized_cap_cents_by_balance_range",
            CohortContext::Addr,
            "realized_cap",
            addr_version,
            |name, source| {
                LazyFiatPerBlockWithDeltas::from_cents_source(
                    name,
                    addr_version,
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
}
