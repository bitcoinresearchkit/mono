mod age_band;
mod compute;
mod horizon;
mod import;
mod vecs;

pub(crate) use age_band::AgeBand;
pub(crate) use brk_cohort::AGE_RANGE_COUNT as AGE_COHORT_COUNT;
pub use horizon::{HorizonId, Horizons};
pub use vecs::{
    AgeRangeVecs, AggregateSources, AggregateVecs, HorizonVecs, Mobility, MobilityId,
    SpendingExposureSeries, Vecs,
};

pub(crate) const HORIZON_COUNT: usize = HorizonId::ALL.len();
pub(crate) const HOURS_PER_DAY: f64 = 24.0;
pub(crate) const MINIMUM_DURATION_DAYS: f64 = 1.0 / HOURS_PER_DAY;
