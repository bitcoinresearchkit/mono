use bitview_cohort::{AgeRange, AgeRangeId};
use vecdb::ColumnId;

const HOURS_PER_DAY: f64 = 24.0;

pub const MINIMUM_DURATION_DAYS: f64 = 1.0 / HOURS_PER_DAY;

#[derive(Clone, Copy, Debug)]
pub struct AgeBand {
    pub lower: f64,
    pub upper: f64,
}

impl AgeBand {
    pub fn all() -> AgeRange<Self> {
        AgeRange::from_fn(|id| {
            let bound = id.bounds();
            Self {
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
    pub fn mobility(exposure: f64) -> f64 {
        if exposure.is_nan() || exposure <= 0.0 {
            0.0
        } else {
            (-(-exposure).exp_m1()).min(1.0 - 1e-12)
        }
    }

    pub fn horizon_mobility(
        hazards: &AgeRange<f64>,
        start_band: AgeRangeId,
        horizon: f64,
        bounds: &AgeRange<Self>,
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
            let duration = if bound.upper.is_finite() {
                remaining.min((bound.upper - age).max(MINIMUM_DURATION_DAYS))
            } else {
                remaining
            };
            exposure += band.select(hazards).max(0.0) * duration;
            remaining -= duration;
            age = bound.upper;
        }

        Self::mobility(exposure)
    }
}
