use bitview_cohort::{AgeAggregate, AgeRange};
use bitview_compute::WeightedCohortState;
use bitview_primitives::{BoundedRatio, Float64, PerDay};

pub(crate) struct PrimaryValues {
    pub(crate) spending_rate: AgeRange<PerDay>,
    pub(crate) spending_exposure: AgeRange<Float64>,
    pub(crate) mobility: AgeRange<BoundedRatio>,
    pub(crate) cohorts: AgeAggregate<WeightedCohortState>,
}
