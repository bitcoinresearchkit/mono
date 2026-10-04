use bitview_cohort::{AgeRange, ByTerm};
use bitview_compute::WeightedCohortState;
use bitview_primitives::{BoundedRatio, Float64, PerDay};

pub(crate) struct PrimaryValues {
    pub(crate) spending_rate: AgeRange<PerDay>,
    pub(crate) spending_exposure: AgeRange<Float64>,
    pub(crate) mobility: AgeRange<BoundedRatio>,
    pub(crate) terms: ByTerm<WeightedCohortState>,
    pub(crate) under_4m: WeightedCohortState,
    pub(crate) under_6m: WeightedCohortState,
    pub(crate) over_4m: WeightedCohortState,
    pub(crate) over_6m: WeightedCohortState,
}
