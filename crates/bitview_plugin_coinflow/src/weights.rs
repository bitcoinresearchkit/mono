use bitview_cohort::{AgeRange, AgeRangeId};
use bitview_compute::{AgeBand, resolve_cohort_value, resolve_cohort_weight};
use brk_types::{Day1, Sats};
use vecdb::{ReadableVec, StorageMode};

use crate::{HorizonId, Horizons, Vecs};

pub(crate) fn horizon_mobilities(
    hazards: &AgeRange<f64>,
    bounds: &AgeRange<AgeBand>,
) -> Horizons<AgeRange<f64>> {
    HorizonId::from_fn(|horizon| {
        AgeRange::from_fn(|age| AgeBand::horizon_mobility(hazards, age, horizon.days(), bounds))
    })
}

impl<M: StorageMode> Vecs<M> {
    /// Daily URPD weight from the age range's lifetime mobility.
    pub fn urpd_weight(&self, age: AgeRangeId, day: Day1, supply: Sats) -> Option<f64> {
        let source = &age.select(&self.age_range.spending_exposure.mobility).day1;
        resolve_cohort_weight(source.collect_one(day).flatten(), supply)
    }

    /// Daily forward spending probabilities across all supported horizons.
    pub fn horizon_weights(
        &self,
        day: Day1,
        supplies: &AgeRange<Sats>,
    ) -> Option<Horizons<AgeRange<f64>>> {
        let hazards = AgeRange::try_from_fn(|age| {
            let source = &age.select(&self.age_range.spending_rate).day1;
            resolve_cohort_value(source.collect_one(day).flatten(), *age.select(supplies))
                .map(|value| value.max(0.0))
                .ok_or(())
        })
        .ok()?;
        Some(horizon_mobilities(&hazards, &AgeBand::all()))
    }
}
