use bitview_cohort::{AGE_RANGE_COUNT, AgeRange, AgeRangeId};

use super::age_band::AgeBand;

#[derive(Clone, Copy)]
pub(super) struct DecayFit {
    slope: f64,
    tau: f64,
    anchor_age: f64,
    anchor_hazard: f64,
}

impl DecayFit {
    /// Fits the observed log hazards of the finite bands (`(hazard, ln hazard)` where complete
    /// and positive) against their ages, weighted by duration.
    fn fit(
        observed: &[Option<(f64, f64)>; AGE_RANGE_COUNT - 1],
        bounds: &AgeRange<AgeBand>,
    ) -> Option<Self> {
        let fitted = || {
            AgeRangeId::ALL
                .iter()
                .zip(observed)
                .filter_map(|(id, observed)| {
                    let band = *id.select(bounds);
                    observed.map(|(hazard, log_hazard)| {
                        let age = (band.lower + band.upper) / 2.0;
                        (band, age, hazard, log_hazard)
                    })
                })
        };
        let mut total_duration = 0.0;
        let mut weighted_age = 0.0;
        let mut weighted_log_hazard = 0.0;
        for (band, age, _, log_hazard) in fitted() {
            let duration = band.upper - band.lower;
            total_duration += duration;
            weighted_age += duration * age;
            weighted_log_hazard += duration * log_hazard;
        }

        if total_duration <= 0.0 {
            return None;
        }

        let mean_age = weighted_age / total_duration;
        let mean_log_hazard = weighted_log_hazard / total_duration;
        let mut covariance = 0.0;
        let mut age_variance = 0.0;
        for (band, age, _, log_hazard) in fitted() {
            let duration = band.upper - band.lower;
            let age_offset = age - mean_age;
            covariance += duration * age_offset * (log_hazard - mean_log_hazard);
            age_variance += duration * age_offset.powi(2);
        }

        if age_variance <= f64::EPSILON {
            return None;
        }

        let slope = covariance / age_variance;
        if slope >= 0.0 {
            return None;
        }

        let tau = -1.0 / slope;
        if !tau.is_finite() || tau <= 0.0 {
            return None;
        }

        let (anchor, _, anchor_hazard, _) = fitted().next_back()?;
        Some(Self {
            slope,
            tau,
            anchor_age: anchor.upper,
            anchor_hazard,
        })
    }

    /// Each band's exposure in one pass: the observed run after every band, summed once from
    /// the oldest finite band down, then each band's occupied part and the fitted tail.
    pub(super) fn exposures(
        hazards: &AgeRange<f64>,
        network_age: f64,
        bounds: &AgeRange<AgeBand>,
    ) -> AgeRange<f64> {
        const FINITE: usize = AGE_RANGE_COUNT - 1;
        let finite = &AgeRangeId::ALL[..FINITE];
        let complete = |id: &AgeRangeId| id.select(bounds).upper <= network_age;
        let observed: [Option<(f64, f64)>; FINITE] = std::array::from_fn(|i| {
            let hazard = *finite[i].select(hazards);
            (complete(&finite[i]) && hazard.is_finite() && hazard > 0.0)
                .then(|| (hazard, hazard.ln()))
        });
        let Some(fit) = Self::fit(&observed, bounds) else {
            return AgeRange::default();
        };

        // From each finite band on, while bands are complete with a measured hazard (a zero adds
        // nothing; an unavailable one ends the run): the exposure of the whole bands and the age
        // the run ends at, none when the band itself ends it.
        let mut runs = [(0.0, None); AGE_RANGE_COUNT];
        for (i, id) in finite.iter().enumerate().rev() {
            let band = *id.select(bounds);
            let hazard = *id.select(hazards);
            if complete(id) && hazard.is_finite() {
                let (after, end) = runs[i + 1];
                runs[i] = (
                    hazard * (band.upper - band.lower) + after,
                    Some(end.unwrap_or(band.upper)),
                );
            }
        }

        AgeRange::from_fn(|start_band| {
            let start = *start_band.select(bounds);
            let (exposure, age) = if start.upper.is_finite() {
                let i = start_band.index();
                let occupied_upper = start.upper.min(network_age.max(start.lower));
                let age = (start.lower + occupied_upper) / 2.0;
                match runs[i].1 {
                    // The start band counts from the midpoint of its occupied part.
                    Some(end) => {
                        let hazard = *start_band.select(hazards);
                        (hazard * (start.upper - age) + runs[i + 1].0, end)
                    }
                    None => (0.0, age),
                }
            } else {
                (0.0, start.lower)
            };
            exposure + fit.tail(hazards, bounds, network_age, age)
        })
    }

    /// The fitted continuation from `age`, anchored on the open-ended band when it is observed.
    fn tail(
        self,
        hazards: &AgeRange<f64>,
        bounds: &AgeRange<AgeBand>,
        network_age: f64,
        age: f64,
    ) -> f64 {
        let tail = bounds.over_15y;
        let tail_hazard = hazards.over_15y;
        let observed_tail =
            network_age > tail.lower && tail_hazard.is_finite() && tail_hazard > 0.0;
        let (anchor_age, anchor_hazard) = if observed_tail {
            (tail.lower, tail_hazard)
        } else {
            (self.anchor_age, self.anchor_hazard)
        };
        let continuation_age = age.max(anchor_age);
        let continuation_hazard =
            anchor_hazard * (self.slope * (continuation_age - anchor_age)).exp();
        continuation_hazard * self.tau
    }
}
