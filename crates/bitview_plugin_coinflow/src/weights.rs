use bitview_cohort::{AgeRange, AgeRangeId};
use bitview_compute::{
    AgeBand, MINIMUM_DURATION_DAYS, resolve_cohort_value, resolve_cohort_weight,
};
use brk_types::{Day1, Sats};
use vecdb::{ReadableVec, StorageMode};

use crate::{HorizonId, Horizons, Vecs};

pub(crate) fn horizon_mobilities(
    hazards: &AgeRange<f64>,
    bounds: &AgeRange<AgeBand>,
) -> Horizons<AgeRange<f64>> {
    let mut weights = HorizonId::from_fn(|_| AgeRange::from_fn(|_| 0.0));
    for &start_band in AgeRangeId::ALL {
        let start = *start_band.select(bounds);
        let mut age = if start.upper.is_finite() {
            (start.lower + start.upper) / 2.0
        } else {
            start.lower
        };
        let mut remaining = HorizonId::ALL.map(HorizonId::days);
        let mut exposures = [0.0; HorizonId::ALL.len()];
        for &band in &AgeRangeId::ALL[start_band.index()..] {
            if remaining.iter().all(|&duration| duration <= 0.0) {
                break;
            }
            let upper = band.select(bounds).upper;
            let duration = (upper - age).max(MINIMUM_DURATION_DAYS);
            let hazard = band.select(hazards).max(0.0);
            for (remaining, exposure) in remaining.iter_mut().zip(&mut exposures) {
                if *remaining > 0.0 {
                    let covered = remaining.min(duration);
                    *exposure += hazard * covered;
                    *remaining -= covered;
                }
            }
            age = upper;
        }
        for (horizon, exposure) in HorizonId::ALL.into_iter().zip(exposures) {
            *start_band.select_mut(horizon.select_mut(&mut weights)) = AgeBand::mobility(exposure);
        }
    }
    weights
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
