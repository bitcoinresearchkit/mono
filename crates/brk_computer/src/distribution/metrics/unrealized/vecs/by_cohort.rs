use std::ops::AddAssign;

use brk_cohort::{CohortContext, UTXOGroupsWithoutAmount};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, PcoVecValue, Rw, StorageMode};

use crate::{
    distribution::metrics::UTXOColumnarMetricWithoutAmount,
    indexes,
    internal::{FiatType, LazyFiatPerBlock},
};

#[derive(Traversable)]
pub struct UnrealizedByCohort<C, M: StorageMode = Rw>
where
    C: FiatType + PcoVecValue,
{
    #[traversable(flatten)]
    pub cohorts: UTXOGroupsWithoutAmount<LazyFiatPerBlock<C>>,
    #[traversable(flatten)]
    pub matrices: UTXOColumnarMetricWithoutAmount<C, M>,
}

impl<C> UnrealizedByCohort<C>
where
    C: FiatType + PcoVecValue + AddAssign,
{
    pub(super) fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        let matrices = UTXOColumnarMetricWithoutAmount::forced_import(
            db,
            &format!("{metric}_cents"),
            version,
        )?;
        let cohorts = UTXOGroupsWithoutAmount::new(|filter, cohort_name| {
            let name = CohortContext::Utxo.metric_name(&filter, cohort_name, metric);
            let source = matrices
                .additive_source(&filter, &format!("{name}_cents"), version)
                .expect("supported unrealized cohort");
            LazyFiatPerBlock::from_boxed_cents_source(&name, version, source, indexes)
        });
        Ok(Self { cohorts, matrices })
    }
}
