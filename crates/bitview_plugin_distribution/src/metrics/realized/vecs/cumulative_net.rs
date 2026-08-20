use brk_error::Result;

use bitview_cohort::{CohortContext, UTXOGroupsWithoutAmountOrType};
use bitview_traversable::Traversable;
use brk_types::{CentsSigned, PartsPerMillionSigned64, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::CumulativeUTXOColumnarMetricWithoutAmountOrType;
use bitview_compute::{CachedWindowStartVec, LazyFiatPerBlockCumulativeWithSumsAndDeltas, Windows};

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
    #[traversable(flatten)]
    pub cumulative: CumulativeUTXOColumnarMetricWithoutAmountOrType<CentsSigned, M>,
}

impl CumulativeNetRealizedByCohort {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let cumulative = CumulativeUTXOColumnarMetricWithoutAmountOrType::forced_import(
            db,
            "net_realized_pnl_cumulative_cents",
            version,
        )?;
        let cohorts = UTXOGroupsWithoutAmountOrType::new(|filter, cohort_name| {
            let name = CohortContext::Utxo.metric_name(&filter, cohort_name, "net_realized_pnl");
            let source = cumulative
                .matrices
                .additive_source(&filter, &format!("{name}_cumulative_cents"), version)
                .expect("supported net realized cohort");
            LazyFiatPerBlockCumulativeWithSumsAndDeltas::from_boxed_cumulative_cents_source(
                &name,
                version,
                source,
                Version::new(5),
                mappings,
                cached_starts,
            )
        });
        Ok(Self {
            cohorts,
            cumulative,
        })
    }
}
