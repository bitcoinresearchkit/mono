use brk_error::Result;

use bitview_traversable::Traversable;
use brk_cohort::{CohortContext, UTXOGroups};
use brk_types::{Cents, Height, Version};
use vecdb::{CachedBoxedVec, Database, Rw, StorageMode};

use crate::metrics::ExactUTXOColumnarMetric;
use bitview_compute::LazyPriceWithRatioPerBlock;

#[derive(Traversable)]
pub struct RealizedPriceByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroups<LazyPriceWithRatioPerBlock>,
    /// Reported in cents per BTC.
    #[traversable(flatten)]
    pub matrices: ExactUTXOColumnarMetric<Cents, M>,
}

impl RealizedPriceByCohort {
    pub fn forced_import(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let matrices = ExactUTXOColumnarMetric::forced_import(db, "realized_price_cents", version)?;
        let cohorts = UTXOGroups::new(|filter, cohort_name| {
            let name = CohortContext::Utxo.metric_name(&filter, cohort_name, "realized_price");
            LazyPriceWithRatioPerBlock::from_boxed_height_source(
                &name,
                version,
                matrices
                    .source(&filter, &format!("{name}_cents"), version)
                    .expect("realized-price cohort source"),
                indexes,
                spot_price,
            )
        });
        Ok(Self { cohorts, matrices })
    }
}
