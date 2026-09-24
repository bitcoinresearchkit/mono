use bitview_cohort::AgeRangeId;
use bitview_compute::resolve_cohort_weight;
use brk_types::{Day1, Sats};
use vecdb::{ReadableVec, StorageMode};

use crate::Vecs;

impl<M: StorageMode> Vecs<M> {
    /// Daily URPD weight from the age range's wakefulness.
    pub fn urpd_weight(&self, age: AgeRangeId, day: Day1, supply: Sats) -> Option<f64> {
        let source = &age.select(&self.age_range.activity.wakefulness).day1;
        resolve_cohort_weight(source.collect_one(day).flatten(), supply)
    }
}
