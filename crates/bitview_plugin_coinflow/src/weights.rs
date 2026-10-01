use bitview_cohort::AgeRangeId;
use bitview_compute::resolve_cohort_weight;
use brk_types::{Height, Sats};
use vecdb::{ReadableVec, StorageMode};

use crate::Vecs;

impl<M: StorageMode> Vecs<M> {
    /// Per-block URPD weight from the age range's lifetime mobility.
    pub fn urpd_weight(&self, age: AgeRangeId, height: Height, supply: Sats) -> Option<f64> {
        let source = &age
            .select(&self.age_range.spending_exposure.mobility)
            .height;
        resolve_cohort_weight(source.collect_one(height), supply)
    }
}
