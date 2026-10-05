use bitview_cohort::AgeAggregate;
use bitview_vecs::IndexSources;
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableBoxedVec};

use super::{Metrics, cohort::CohortMetrics};

impl Metrics {
    pub fn import(
        db: &Database,
        owner: &str,
        version: Version,
        indexes: &IndexSources,
        spot: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        Ok(Self {
            replay: Default::default(),
            cohorts: AgeAggregate::try_from_fn(|id| {
                CohortMetrics::import(db, owner, id, version, indexes, spot)
            })?,
        })
    }
}
