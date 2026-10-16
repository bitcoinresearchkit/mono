use bitview_cohort::{AgeRange, AgeRangeId};
use bitview_primitives::{BoundedRatio, CentsSquaredSats};
use brk_types::{CentsSats, Height, Sats};
use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use vecdb::{ReadableVec, VecValue};

use crate::WeightedCohortState;

/// Bounded inputs shared by the weighted age models. Each consumer owns its batch.
#[derive(Default)]
pub struct CohortAccounting {
    pub supplies: AgeRange<Vec<Sats>>,
    pub loss_supplies: AgeRange<Vec<Sats>>,
    pub cap_raw: AgeRange<Vec<CentsSats>>,
    pub capitalized_cap_raw: AgeRange<Vec<CentsSquaredSats>>,
}

impl CohortAccounting {
    #[inline]
    pub fn weighted(
        &self,
        age: AgeRangeId,
        offset: usize,
        weight: BoundedRatio,
    ) -> WeightedCohortState {
        WeightedCohortState::new(
            age.select(&self.supplies)[offset],
            age.select(&self.loss_supplies)[offset],
            age.select(&self.cap_raw)[offset],
            age.select(&self.capitalized_cap_raw)[offset],
            weight,
        )
    }
}

/// Collect age columns into buffers whose capacity survives the next chunk.
pub fn collect_age_range<T, V>(
    sources: &AgeRange<&V>,
    targets: &mut AgeRange<Vec<T>>,
    start: usize,
    end: usize,
) where
    T: VecValue,
    V: ReadableVec<Height, T> + ?Sized,
{
    sources
        .as_array()
        .into_par_iter()
        .zip(targets.as_array_mut().into_par_iter())
        .for_each(|(source, target)| source.collect_range_into_at(start, end, target));
}
