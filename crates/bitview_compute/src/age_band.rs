use bitview_cohort::{AgeRange, AgeRangeId};

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
}
