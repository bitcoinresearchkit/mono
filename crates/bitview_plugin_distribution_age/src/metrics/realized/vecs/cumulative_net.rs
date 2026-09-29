use bitview_cohort::{CohortContext, UTXOGroupsWithoutAmountOrType};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFiatPerBlockCumulativeWithSumsAndDeltas, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{CentsSigned, PartsPerMillionSigned64, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::CumulativeUTXOCoreSources;

#[derive(Traversable)]
pub struct CumulativeNetRealizedByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroupsWithoutAmountOrType<
        LazyFiatPerBlockCumulativeWithSumsAndDeltas<
            CentsSigned,
            CentsSigned,
            PartsPerMillionSigned64,
        >,
    >,
    #[traversable(hidden)]
    pub stored: CumulativeUTXOCoreSources<CentsSigned, M>,
}

impl CumulativeNetRealizedByCohort {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let stored = CumulativeUTXOCoreSources::forced_import(
            db,
            "net_realized_pnl_cumulative_cents",
            version,
        )?;
        let cohorts = UTXOGroupsWithoutAmountOrType::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "net_realized_pnl");
            let source = stored
                .stored
                .get(cohort_id)
                .expect("supported net realized cohort");
            LazyFiatPerBlockCumulativeWithSumsAndDeltas::from_cumulative_cents_source(
                &name,
                version,
                source,
                Version::new(5),
                mappings,
                window_starts,
            )
        });
        Ok(Self { cohorts, stored })
    }
}
