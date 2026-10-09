use bitview_cohort::AgeRangeId;
use bitview_compute::resolve_cohort_weight;
use bitview_primitives::BoundedRatio;
use brk_types::{Height, Sats};
use vecdb::{ReadableVec, StorageMode};

use crate::Vecs;

impl<M: StorageMode> Vecs<M> {
    /// Per-block URPD weight from the age range's wakefulness.
    pub fn urpd_weight(&self, age: AgeRangeId, height: Height, supply: Sats) -> Option<f64> {
        let sources = self.age_ranges.urpd_weight_sources();
        resolve_cohort_weight(age.select(&sources).collect_one(height), supply)
    }

    /// All-supply awake in-loss share at full precision, for models calibrating on it.
    pub fn all_awake_supply_in_loss_share(&self) -> &impl ReadableVec<Height, BoundedRatio> {
        &self.aggregate.sources.awake_supply_in_loss_share.all
    }
}
