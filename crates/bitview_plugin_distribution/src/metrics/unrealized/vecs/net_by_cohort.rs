use brk_error::Result;

use bitview_traversable::Traversable;
use brk_cohort::{CohortContext, UTXOGroupsWithoutAmountOrType};
use brk_types::{CentsSigned, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::UTXOColumnarMetricWithoutAmountOrType;
use bitview_compute::LazyFiatPerBlock;

#[derive(Traversable)]
pub struct NetUnrealizedByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroupsWithoutAmountOrType<LazyFiatPerBlock<CentsSigned>>,
    #[traversable(flatten)]
    pub matrices: UTXOColumnarMetricWithoutAmountOrType<CentsSigned, M>,
}

impl NetUnrealizedByCohort {
    pub fn forced_import(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
    ) -> Result<Self> {
        let metric = "net_unrealized_pnl";
        let matrices = UTXOColumnarMetricWithoutAmountOrType::forced_import(
            db,
            "net_unrealized_pnl_cents",
            version,
        )?;
        let cohorts = UTXOGroupsWithoutAmountOrType::new(|filter, cohort_name| {
            let name = CohortContext::Utxo.metric_name(&filter, cohort_name, metric);
            let source = matrices
                .additive_source(&filter, &format!("{name}_cents"), version)
                .expect("supported net unrealized cohort");
            LazyFiatPerBlock::from_boxed_cents_source(&name, version, source, indexes)
        });
        Ok(Self { cohorts, matrices })
    }
}
