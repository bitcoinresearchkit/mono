use bitview_cohort::{AgeRange, AgeRangeId};
use bitview_compute::{AgeBand, MINIMUM_DURATION_DAYS};

use crate::AGE_COHORT_COUNT;

#[derive(Clone, Copy)]
pub(super) struct DecayFit {
    slope: f64,
    tau: f64,
    anchor_age: f64,
    anchor_hazard: f64,
}

impl DecayFit {
    fn fit(hazards: &AgeRange<f64>, network_age: f64, bounds: &AgeRange<AgeBand>) -> Option<Self> {
        let mut total_duration = 0.0;
        let mut weighted_age = 0.0;
        let mut weighted_log_hazard = 0.0;
        let mut anchor = None;

        for &id in &AgeRangeId::ALL[..AGE_COHORT_COUNT - 1] {
            let band = *id.select(bounds);
            let hazard = *id.select(hazards);
            if band.upper > network_age || !hazard.is_finite() || hazard <= 0.0 {
                continue;
            }

            let age = (band.lower + band.upper) / 2.0;
            let duration = band.upper - band.lower;
            let log_hazard = hazard.ln();
            total_duration += duration;
            weighted_age += duration * age;
            weighted_log_hazard += duration * log_hazard;
            anchor = Some((band.upper, hazard));
        }

        if total_duration <= 0.0 {
            return None;
        }

        let mean_age = weighted_age / total_duration;
        let mean_log_hazard = weighted_log_hazard / total_duration;
        let mut covariance = 0.0;
        let mut age_variance = 0.0;

        for &id in &AgeRangeId::ALL[..AGE_COHORT_COUNT - 1] {
            let band = *id.select(bounds);
            let hazard = *id.select(hazards);
            if band.upper > network_age || !hazard.is_finite() || hazard <= 0.0 {
                continue;
            }

            let age = (band.lower + band.upper) / 2.0;
            let duration = band.upper - band.lower;
            let log_hazard = hazard.ln();
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

        let (anchor_age, anchor_hazard) = anchor?;
        Some(Self {
            slope,
            tau,
            anchor_age,
            anchor_hazard,
        })
    }

    pub(super) fn exposures(
        hazards: &AgeRange<f64>,
        network_age: f64,
        bounds: &AgeRange<AgeBand>,
    ) -> AgeRange<f64> {
        let Some(fit) = Self::fit(hazards, network_age, bounds) else {
            return AgeRange::default();
        };

        AgeRange::from_fn(|start_band| fit.exposure(hazards, start_band, network_age, bounds))
    }

    fn exposure(
        self,
        hazards: &AgeRange<f64>,
        start_band: AgeRangeId,
        network_age: f64,
        bounds: &AgeRange<AgeBand>,
    ) -> f64 {
        let start = *start_band.select(bounds);
        let occupied_upper = if start.upper.is_finite() {
            start.upper.min(network_age.max(start.lower))
        } else {
            start.lower
        };
        let mut age = if start.upper.is_finite() {
            (start.lower + occupied_upper) / 2.0
        } else {
            start.lower
        };
        let mut exposure = 0.0;

        for &band_id in &AgeRangeId::ALL[start_band.index()..AGE_COHORT_COUNT - 1] {
            let band = *band_id.select(bounds);
            let duration = (band.upper - age.max(band.lower)).max(MINIMUM_DURATION_DAYS);
            let hazard = *band_id.select(hazards);
            let observed = band.upper <= network_age && hazard.is_finite() && hazard > 0.0;
            if !observed {
                break;
            }

            exposure += hazard * duration;
            age = band.upper;
        }

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
        exposure + (continuation_hazard * self.tau).max(0.0)
    }
}
