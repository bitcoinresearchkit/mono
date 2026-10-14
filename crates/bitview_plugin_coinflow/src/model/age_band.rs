use bitview_cohort::{AgeRange, AgeRangeId};

const HOURS_PER_DAY: f64 = 24.0;

pub(super) const MINIMUM_DURATION_DAYS: f64 = 1.0 / HOURS_PER_DAY;

#[derive(Clone, Copy, Debug)]
pub(crate) struct AgeBand {
    pub(super) lower: f64,
    pub(super) upper: f64,
}

impl AgeBand {
    pub(crate) fn all() -> AgeRange<Self> {
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

    /// Exposures are defined: the decay fit gives zeros without a fit and sums finite terms.
    #[inline]
    pub(super) fn mobility(exposure: f64) -> f64 {
        debug_assert!(!exposure.is_nan(), "undefined exposure");
        if exposure <= 0.0 {
            0.0
        } else {
            (-(-exposure).exp_m1()).min(1.0 - 1e-12)
        }
    }
}
