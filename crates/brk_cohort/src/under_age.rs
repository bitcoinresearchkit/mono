use brk_traversable::Traversable;
use rayon::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;

use super::{
    CohortName, Filter, HOURS_1M, HOURS_1W, HOURS_1Y, HOURS_2M, HOURS_2Y, HOURS_3M, HOURS_3Y,
    HOURS_4M, HOURS_4Y, HOURS_5M, HOURS_5Y, HOURS_6M, HOURS_6Y, HOURS_7Y, HOURS_8Y, HOURS_9M,
    HOURS_10Y, HOURS_12Y, HOURS_15Y, HOURS_18M, TimeFilter,
};

/// Under-age thresholds in hours
pub const UNDER_AGE_HOURS: UnderAge<usize> = UnderAge {
    _1w: HOURS_1W,
    _1m: HOURS_1M,
    _2m: HOURS_2M,
    _3m: HOURS_3M,
    _4m: HOURS_4M,
    _5m: HOURS_5M,
    _6m: HOURS_6M,
    _9m: HOURS_9M,
    _1y: HOURS_1Y,
    _18m: HOURS_18M,
    _2y: HOURS_2Y,
    _3y: HOURS_3Y,
    _4y: HOURS_4Y,
    _5y: HOURS_5Y,
    _6y: HOURS_6Y,
    _7y: HOURS_7Y,
    _8y: HOURS_8Y,
    _10y: HOURS_10Y,
    _12y: HOURS_12Y,
    _15y: HOURS_15Y,
};

/// Under-age filters (LowerThan threshold in hours)
pub const UNDER_AGE_FILTERS: UnderAge<Filter> = UnderAge {
    _1w: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._1w)),
    _1m: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._1m)),
    _2m: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._2m)),
    _3m: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._3m)),
    _4m: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._4m)),
    _5m: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._5m)),
    _6m: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._6m)),
    _9m: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._9m)),
    _1y: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._1y)),
    _18m: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._18m)),
    _2y: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._2y)),
    _3y: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._3y)),
    _4y: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._4y)),
    _5y: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._5y)),
    _6y: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._6y)),
    _7y: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._7y)),
    _8y: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._8y)),
    _10y: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._10y)),
    _12y: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._12y)),
    _15y: Filter::Time(TimeFilter::LowerThan(UNDER_AGE_HOURS._15y)),
};

/// Under-age names
pub const UNDER_AGE_NAMES: UnderAge<CohortName> = UnderAge {
    _1w: CohortName::new("under_1w_old", "<1w", "Under 1 Week Old"),
    _1m: CohortName::new("under_1m_old", "<1m", "Under 1 Month Old"),
    _2m: CohortName::new("under_2m_old", "<2m", "Under 2 Months Old"),
    _3m: CohortName::new("under_3m_old", "<3m", "Under 3 Months Old"),
    _4m: CohortName::new("under_4m_old", "<4m", "Under 4 Months Old"),
    _5m: CohortName::new("under_5m_old", "<5m", "Under 5 Months Old"),
    _6m: CohortName::new("under_6m_old", "<6m", "Under 6 Months Old"),
    _9m: CohortName::new("under_9m_old", "<9m", "Under 9 Months Old"),
    _1y: CohortName::new("under_1y_old", "<1y", "Under 1 Year Old"),
    _18m: CohortName::new("under_18m_old", "<18m", "Under 18 Months Old"),
    _2y: CohortName::new("under_2y_old", "<2y", "Under 2 Years Old"),
    _3y: CohortName::new("under_3y_old", "<3y", "Under 3 Years Old"),
    _4y: CohortName::new("under_4y_old", "<4y", "Under 4 Years Old"),
    _5y: CohortName::new("under_5y_old", "<5y", "Under 5 Years Old"),
    _6y: CohortName::new("under_6y_old", "<6y", "Under 6 Years Old"),
    _7y: CohortName::new("under_7y_old", "<7y", "Under 7 Years Old"),
    _8y: CohortName::new("under_8y_old", "<8y", "Under 8 Years Old"),
    _10y: CohortName::new("under_10y_old", "<10y", "Under 10 Years Old"),
    _12y: CohortName::new("under_12y_old", "<12y", "Under 12 Years Old"),
    _15y: CohortName::new("under_15y_old", "<15y", "Under 15 Years Old"),
};

#[derive(Debug, Default, Clone, Traversable, Serialize, JsonSchema)]
pub struct UnderAge<T> {
    pub _1w: T,
    pub _1m: T,
    pub _2m: T,
    pub _3m: T,
    pub _4m: T,
    pub _5m: T,
    pub _6m: T,
    pub _9m: T,
    pub _1y: T,
    pub _18m: T,
    pub _2y: T,
    pub _3y: T,
    pub _4y: T,
    pub _5y: T,
    pub _6y: T,
    pub _7y: T,
    pub _8y: T,
    pub _10y: T,
    pub _12y: T,
    pub _15y: T,
}

define_column_id!(
    UnderAgeId for UnderAge, version = 1 {
        Under1W => _1w,
        Under1M => _1m,
        Under2M => _2m,
        Under3M => _3m,
        Under4M => _4m,
        Under5M => _5m,
        Under6M => _6m,
        Under9M => _9m,
        Under1Y => _1y,
        Under18M => _18m,
        Under2Y => _2y,
        Under3Y => _3y,
        Under4Y => _4y,
        Under5Y => _5y,
        Under6Y => _6y,
        Under7Y => _7y,
        Under8Y => _8y,
        Under10Y => _10y,
        Under12Y => _12y,
        Under15Y => _15y,
    }
);

impl UnderAge<CohortName> {
    pub const fn names() -> &'static Self {
        &UNDER_AGE_NAMES
    }
}

impl<T> UnderAge<T> {
    pub fn new<F>(mut create: F) -> Self
    where
        F: FnMut(Filter, &'static str) -> T,
    {
        let f = UNDER_AGE_FILTERS;
        let n = UNDER_AGE_NAMES;
        Self {
            _1w: create(f._1w.clone(), n._1w.id),
            _1m: create(f._1m.clone(), n._1m.id),
            _2m: create(f._2m.clone(), n._2m.id),
            _3m: create(f._3m.clone(), n._3m.id),
            _4m: create(f._4m.clone(), n._4m.id),
            _5m: create(f._5m.clone(), n._5m.id),
            _6m: create(f._6m.clone(), n._6m.id),
            _9m: create(f._9m.clone(), n._9m.id),
            _1y: create(f._1y.clone(), n._1y.id),
            _18m: create(f._18m.clone(), n._18m.id),
            _2y: create(f._2y.clone(), n._2y.id),
            _3y: create(f._3y.clone(), n._3y.id),
            _4y: create(f._4y.clone(), n._4y.id),
            _5y: create(f._5y.clone(), n._5y.id),
            _6y: create(f._6y.clone(), n._6y.id),
            _7y: create(f._7y.clone(), n._7y.id),
            _8y: create(f._8y.clone(), n._8y.id),
            _10y: create(f._10y.clone(), n._10y.id),
            _12y: create(f._12y.clone(), n._12y.id),
            _15y: create(f._15y.clone(), n._15y.id),
        }
    }

    pub fn try_new<F, E>(mut create: F) -> Result<Self, E>
    where
        F: FnMut(Filter, &'static str) -> Result<T, E>,
    {
        let f = UNDER_AGE_FILTERS;
        let n = UNDER_AGE_NAMES;
        Ok(Self {
            _1w: create(f._1w.clone(), n._1w.id)?,
            _1m: create(f._1m.clone(), n._1m.id)?,
            _2m: create(f._2m.clone(), n._2m.id)?,
            _3m: create(f._3m.clone(), n._3m.id)?,
            _4m: create(f._4m.clone(), n._4m.id)?,
            _5m: create(f._5m.clone(), n._5m.id)?,
            _6m: create(f._6m.clone(), n._6m.id)?,
            _9m: create(f._9m.clone(), n._9m.id)?,
            _1y: create(f._1y.clone(), n._1y.id)?,
            _18m: create(f._18m.clone(), n._18m.id)?,
            _2y: create(f._2y.clone(), n._2y.id)?,
            _3y: create(f._3y.clone(), n._3y.id)?,
            _4y: create(f._4y.clone(), n._4y.id)?,
            _5y: create(f._5y.clone(), n._5y.id)?,
            _6y: create(f._6y.clone(), n._6y.id)?,
            _7y: create(f._7y.clone(), n._7y.id)?,
            _8y: create(f._8y.clone(), n._8y.id)?,
            _10y: create(f._10y.clone(), n._10y.id)?,
            _12y: create(f._12y.clone(), n._12y.id)?,
            _15y: create(f._15y.clone(), n._15y.id)?,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [
            &self._1w, &self._1m, &self._2m, &self._3m, &self._4m, &self._5m, &self._6m, &self._9m,
            &self._1y, &self._18m, &self._2y, &self._3y, &self._4y, &self._5y, &self._6y,
            &self._7y, &self._8y, &self._10y, &self._12y, &self._15y,
        ]
        .into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [
            &mut self._1w,
            &mut self._1m,
            &mut self._2m,
            &mut self._3m,
            &mut self._4m,
            &mut self._5m,
            &mut self._6m,
            &mut self._9m,
            &mut self._1y,
            &mut self._18m,
            &mut self._2y,
            &mut self._3y,
            &mut self._4y,
            &mut self._5y,
            &mut self._6y,
            &mut self._7y,
            &mut self._8y,
            &mut self._10y,
            &mut self._12y,
            &mut self._15y,
        ]
        .into_iter()
    }

    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        [
            &mut self._1w,
            &mut self._1m,
            &mut self._2m,
            &mut self._3m,
            &mut self._4m,
            &mut self._5m,
            &mut self._6m,
            &mut self._9m,
            &mut self._1y,
            &mut self._18m,
            &mut self._2y,
            &mut self._3y,
            &mut self._4y,
            &mut self._5y,
            &mut self._6y,
            &mut self._7y,
            &mut self._8y,
            &mut self._10y,
            &mut self._12y,
            &mut self._15y,
        ]
        .into_par_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AGE_RANGE_FILTERS;

    #[test]
    fn new_thresholds_include_only_younger_ranges() {
        assert!(UNDER_AGE_FILTERS._9m.includes(&AGE_RANGE_FILTERS._6m_to_9m));
        assert!(!UNDER_AGE_FILTERS._9m.includes(&AGE_RANGE_FILTERS._9m_to_1y));
        assert!(
            UNDER_AGE_FILTERS
                ._18m
                .includes(&AGE_RANGE_FILTERS._1y_to_18m)
        );
        assert!(
            !UNDER_AGE_FILTERS
                ._18m
                .includes(&AGE_RANGE_FILTERS._18m_to_2y)
        );
    }
}
