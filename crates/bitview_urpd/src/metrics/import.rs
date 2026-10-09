use bitview_cohort::AgeAggregate;
use bitview_vecs::IndexSources;
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::{Metrics, cohort::CohortMetrics};

impl Metrics {
    pub fn import(
        db: &Database,
        owner: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let version = version + Version::TWO;
        Ok(Self {
            replay: Default::default(),
            cohorts: AgeAggregate::try_from_fn(|id| {
                CohortMetrics::import(db, owner, id, version, indexes)
            })?,
        })
    }
}
