use bitview_cohort::AgeRangeId;
use bitview_compute::resolve_cohort_weight;
use brk_types::{Height, Sats};
use vecdb::{ReadableVec, StorageMode};

use crate::Vecs;

impl<M: StorageMode> Vecs<M> {
    /// Per-block URPD weight from the age range's wakefulness.
    pub fn urpd_weight(&self, age: AgeRangeId, height: Height, supply: Sats) -> Option<f64> {
        let sources = self.age_range.urpd_weight_sources();
        resolve_cohort_weight(age.select(&sources).collect_one(height), supply)
    }
}
