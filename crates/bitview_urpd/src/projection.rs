use bitview_cohort::{AGE_RANGE_COUNT, AGE_RANGE_IDS, AgeRange, AgeRangeId};
use bitview_primitives::CentsCompact;
use brk_types::Sats;

use crate::ProjectedBucket;

/// Per-block weights and filters in the histogram's age order. Bucket projection
/// multiplies each occupied age once per model, rounding only after aggregation.
pub(crate) struct Projection<const N: usize, const C: usize> {
    weights: [[f64; N]; AGE_RANGE_COUNT],
    cohorts: [u32; C],
    ages: u32,
}

impl<const N: usize, const C: usize> Projection<N, C> {
    pub fn new(weights: &[Option<&AgeRange<f64>>; N], cohorts: [&[AgeRangeId]; C]) -> Self {
        let cohorts =
            cohorts.map(|cohort| cohort.iter().fold(0, |mask, age| mask | (1 << age.index())));
        let ages = cohorts.iter().fold(0, |union, mask| union | mask);
        Self {
            weights: AGE_RANGE_IDS
                .map(|age| weights.map(|weights| weights.map_or(0.0, |w| *age.select(w)))),
            cohorts,
            ages,
        }
    }

    pub fn bucket(
        &self,
        price: CentsCompact,
        amounts: &[Vec<u64>; AGE_RANGE_COUNT],
        slot: usize,
        occupied: u32,
    ) -> Option<ProjectedBucket<N, C>> {
        let mut occupied = occupied & self.ages;
        if occupied == 0 {
            return None;
        }
        let mut raw = [Sats::ZERO; C];
        let mut masses = [[0.0_f64; N]; C];
        while occupied != 0 {
            let age = occupied.trailing_zeros() as usize;
            occupied &= occupied - 1;
            let sats = amounts[age][slot];
            let weights = &self.weights[age];
            let age_bit = 1 << age;
            let contribution = weights.map(|weight| sats as f64 * weight);
            for ((total, masses), &cohort) in raw.iter_mut().zip(&mut masses).zip(&self.cohorts) {
                if cohort & age_bit != 0 {
                    *total += Sats::new(sats);
                    for (total, mass) in masses.iter_mut().zip(contribution) {
                        *total += mass;
                    }
                }
            }
        }
        let mut weighted = [[Sats::ZERO; N]; C];
        for (target, masses) in weighted.iter_mut().zip(masses) {
            for (target, mass) in target.iter_mut().zip(masses) {
                // Conversion already floors positive mass to whole sats.
                *target = Sats::new(mass as u64);
            }
        }
        Some(ProjectedBucket {
            price,
            raw,
            weighted,
        })
    }
}
