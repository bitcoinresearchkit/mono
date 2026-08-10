mod compute;
mod import;
mod vecs;

use brk_cohort::{AgeRange, AgeRangeId};
use vecdb::ColumnId;

pub(crate) use brk_cohort::AGE_RANGE_COUNT as AGE_COHORT_COUNT;
pub(crate) use vecs::HORIZON_DAYS;
pub use vecs::{
    AgeRangeVecs, AggregateVecs, HorizonVecs, Horizons, SpendingExposureSeries, Split, Vecs,
};

pub(crate) const HORIZON_COUNT: usize = 7;
pub(crate) const HOURS_PER_DAY: f64 = 24.0;
pub(crate) const MINIMUM_DURATION_DAYS: f64 = 1.0 / HOURS_PER_DAY;

#[derive(Clone, Copy, Debug)]
pub(crate) struct AgeBand {
    pub lower: f64,
    pub upper: f64,
}

pub(crate) fn age_bounds_days() -> AgeRange<AgeBand> {
    AgeRange::from_fn(|id| {
        let bound = id.bounds();
        AgeBand {
            lower: bound.start as f64 / HOURS_PER_DAY,
            upper: if id == AgeRangeId::Over15Y {
                f64::INFINITY
            } else {
                bound.end as f64 / HOURS_PER_DAY
            },
        }
    })
}

#[inline]
pub(crate) fn mobility(exposure: f64) -> f64 {
    if exposure.is_nan() || exposure <= 0.0 {
        0.0
    } else {
        (-(-exposure).exp_m1()).min(1.0 - 1e-12)
    }
}

pub(crate) fn horizon_mobility(
    hazards: &AgeRange<f64>,
    start_band: AgeRangeId,
    horizon: f64,
    bounds: &AgeRange<AgeBand>,
) -> f64 {
    let start = *start_band.select(bounds);
    let mut age = if start.upper.is_finite() {
        (start.lower + start.upper) / 2.0
    } else {
        start.lower
    };
    let mut remaining = horizon;
    let mut exposure = 0.0;

    for &band in &AgeRangeId::ALL[start_band.index()..] {
        if remaining <= 0.0 {
            break;
        }
        let bound = *band.select(bounds);
        let upper = bound.upper;
        let duration = if upper.is_finite() {
            remaining.min((upper - age).max(MINIMUM_DURATION_DAYS))
        } else {
            remaining
        };
        exposure += band.select(hazards).max(0.0) * duration;
        remaining -= duration;
        age = upper;
    }

    mobility(exposure)
}
