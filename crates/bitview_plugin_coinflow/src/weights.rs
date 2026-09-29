use bitview_cohort::{AgeRange, AgeRangeId};
use bitview_compute::{
    AgeBand, MINIMUM_DURATION_DAYS, resolve_cohort_value, resolve_cohort_weight,
};
use brk_types::{Height, Sats};
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
    /// Per-block URPD weight from the age range's lifetime mobility.
    pub fn urpd_weight(&self, age: AgeRangeId, height: Height, supply: Sats) -> Option<f64> {
        let source = &age
            .select(&self.age_range.spending_exposure.mobility)
            .height;
        resolve_cohort_weight(source.collect_one(height), supply)
    }

    /// Per-block forward spending probabilities across all supported horizons.
    pub fn horizon_weights(
        &self,
        height: Height,
        supplies: &AgeRange<Sats>,
    ) -> Option<Horizons<AgeRange<f64>>> {
        let hazards = AgeRange::try_from_fn(|age| {
            let source = &age.select(&self.age_range.spending_rate).height;
            resolve_cohort_value(source.collect_one(height), *age.select(supplies))
                .map(|value| value.max(0.0))
                .ok_or(())
        })
        .ok()?;
        Some(horizon_mobilities(&hazards, &AgeBand::all()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_horizon_compounds_hazards_across_age_ranges() {
        let bounds = AgeBand::all();
        let hazards = AgeRange::from_fn(|_| 0.01);
        let probabilities = horizon_mobilities(&hazards, &bounds);
        for horizon in HorizonId::ALL {
            let probability = *AgeRangeId::From1DTo1W.select(horizon.select(&probabilities));
            assert!((probability - AgeBand::mobility(0.01 * horizon.days())).abs() < 1e-12);
        }
        let hazards = AgeRange::from_fn(|age| match age {
            AgeRangeId::From3MTo4M => 0.01,
            AgeRangeId::From4MTo5M => 0.02,
            AgeRangeId::From5MTo6M => 0.03,
            AgeRangeId::From6MTo9M => 0.04,
            _ => 0.0,
        });
        let probabilities = horizon_mobilities(&hazards, &bounds);
        // Start at day 105: 15 days in the first band, then complete and partial bands.
        for (horizon, exposure) in [(HorizonId::M1, 0.45), (HorizonId::M3, 2.25)] {
            let probability = *AgeRangeId::From3MTo4M.select(horizon.select(&probabilities));
            assert!((probability - AgeBand::mobility(exposure)).abs() < 1e-12);
        }
    }
}
