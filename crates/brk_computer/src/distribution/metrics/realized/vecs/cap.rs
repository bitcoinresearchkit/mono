use brk_cohort::{CohortContext, UTXOGroups};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, CentsSigned, PartsPerMillionSigned64, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::{
    distribution::metrics::UTXOColumnarMetric,
    indexes,
    internal::{CachedWindowStartVec, LazyFiatPerBlockWithDeltas, Windows},
};

#[derive(Traversable)]
pub struct RealizedCapByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts:
        UTXOGroups<LazyFiatPerBlockWithDeltas<Cents, CentsSigned, PartsPerMillionSigned64>>,
    #[traversable(flatten)]
    pub matrices: UTXOColumnarMetric<Cents, M>,
}

impl RealizedCapByCohort {
    pub(super) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let matrices = UTXOColumnarMetric::forced_import(db, "realized_cap_cents", version)?;
        let cohorts = UTXOGroups::new(|filter, cohort_name| {
            let name = CohortContext::Utxo.metric_name(&filter, cohort_name, "realized_cap");
            LazyFiatPerBlockWithDeltas::from_boxed_cents_source(
                &name,
                version,
                matrices
                    .additive_source(&filter, &format!("{name}_cents"), version)
                    .expect("realized-cap cohort source"),
                Version::TWO,
                indexes,
                cached_starts,
            )
        });
        Ok(Self { cohorts, matrices })
    }
}
