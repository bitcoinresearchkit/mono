use brk_traversable::Traversable;
use rayon::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;

use super::{
    CohortName, Filter, HOURS_1D, HOURS_1M, HOURS_1W, HOURS_1Y, HOURS_2M, HOURS_2Y, HOURS_3M,
    HOURS_3Y, HOURS_4M, HOURS_4Y, HOURS_5M, HOURS_5Y, HOURS_6M, HOURS_6Y, HOURS_7Y, HOURS_8Y,
    HOURS_9M, HOURS_10Y, HOURS_12Y, HOURS_18M, TimeFilter,
};

/// Over-age thresholds in hours
pub const OVER_AGE_HOURS: OverAge<usize> = OverAge {
    _1d: HOURS_1D,
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
};

/// Over-age filters (GreaterOrEqual threshold in hours)
pub const OVER_AGE_FILTERS: OverAge<Filter> = OverAge {
    _1d: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._1d)),
    _1w: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._1w)),
    _1m: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._1m)),
    _2m: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._2m)),
    _3m: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._3m)),
    _4m: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._4m)),
    _5m: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._5m)),
    _6m: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._6m)),
    _9m: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._9m)),
    _1y: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._1y)),
    _18m: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._18m)),
    _2y: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._2y)),
    _3y: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._3y)),
    _4y: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._4y)),
    _5y: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._5y)),
    _6y: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._6y)),
    _7y: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._7y)),
    _8y: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._8y)),
    _10y: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._10y)),
    _12y: Filter::Time(TimeFilter::GreaterOrEqual(OVER_AGE_HOURS._12y)),
};

/// Over-age names
pub const OVER_AGE_NAMES: OverAge<CohortName> = OverAge {
    _1d: CohortName::new("over_1d_old", "1d+", "Over 1 Day Old"),
    _1w: CohortName::new("over_1w_old", "1w+", "Over 1 Week Old"),
    _1m: CohortName::new("over_1m_old", "1m+", "Over 1 Month Old"),
    _2m: CohortName::new("over_2m_old", "2m+", "Over 2 Months Old"),
    _3m: CohortName::new("over_3m_old", "3m+", "Over 3 Months Old"),
    _4m: CohortName::new("over_4m_old", "4m+", "Over 4 Months Old"),
    _5m: CohortName::new("over_5m_old", "5m+", "Over 5 Months Old"),
    _6m: CohortName::new("over_6m_old", "6m+", "Over 6 Months Old"),
    _9m: CohortName::new("over_9m_old", "9m+", "Over 9 Months Old"),
    _1y: CohortName::new("over_1y_old", "1y+", "Over 1 Year Old"),
    _18m: CohortName::new("over_18m_old", "18m+", "Over 18 Months Old"),
    _2y: CohortName::new("over_2y_old", "2y+", "Over 2 Years Old"),
    _3y: CohortName::new("over_3y_old", "3y+", "Over 3 Years Old"),
    _4y: CohortName::new("over_4y_old", "4y+", "Over 4 Years Old"),
    _5y: CohortName::new("over_5y_old", "5y+", "Over 5 Years Old"),
    _6y: CohortName::new("over_6y_old", "6y+", "Over 6 Years Old"),
    _7y: CohortName::new("over_7y_old", "7y+", "Over 7 Years Old"),
    _8y: CohortName::new("over_8y_old", "8y+", "Over 8 Years Old"),
    _10y: CohortName::new("over_10y_old", "10y+", "Over 10 Years Old"),
    _12y: CohortName::new("over_12y_old", "12y+", "Over 12 Years Old"),
};

#[derive(Debug, Default, Clone, Traversable, Serialize, JsonSchema)]
pub struct OverAge<T> {
    pub _1d: T,
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
}

define_column_id!(
    OverAgeId for OverAge, version = 1 {
        Over1D => _1d,
        Over1W => _1w,
        Over1M => _1m,
        Over2M => _2m,
        Over3M => _3m,
        Over4M => _4m,
        Over5M => _5m,
        Over6M => _6m,
        Over9M => _9m,
        Over1Y => _1y,
        Over18M => _18m,
        Over2Y => _2y,
        Over3Y => _3y,
        Over4Y => _4y,
        Over5Y => _5y,
        Over6Y => _6y,
        Over7Y => _7y,
        Over8Y => _8y,
        Over10Y => _10y,
        Over12Y => _12y,
    }
);

impl OverAge<CohortName> {
    pub const fn names() -> &'static Self {
        &OVER_AGE_NAMES
    }
}

impl<T> OverAge<T> {
    pub fn new<F>(mut create: F) -> Self
    where
        F: FnMut(Filter, &'static str) -> T,
    {
        let f = OVER_AGE_FILTERS;
        let n = OVER_AGE_NAMES;
        Self {
            _1d: create(f._1d.clone(), n._1d.id),
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
        }
    }

    pub fn try_new<F, E>(mut create: F) -> Result<Self, E>
    where
        F: FnMut(Filter, &'static str) -> Result<T, E>,
    {
        let f = OVER_AGE_FILTERS;
        let n = OVER_AGE_NAMES;
        Ok(Self {
            _1d: create(f._1d.clone(), n._1d.id)?,
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
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [
            &self._1d, &self._1w, &self._1m, &self._2m, &self._3m, &self._4m, &self._5m, &self._6m,
            &self._9m, &self._1y, &self._18m, &self._2y, &self._3y, &self._4y, &self._5y,
            &self._6y, &self._7y, &self._8y, &self._10y, &self._12y,
        ]
        .into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [
            &mut self._1d,
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
        ]
        .into_iter()
    }

    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        [
            &mut self._1d,
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
        ]
        .into_par_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AGE_RANGE_FILTERS;

    #[test]
    fn new_thresholds_include_only_older_ranges() {
        assert!(!OVER_AGE_FILTERS._9m.includes(&AGE_RANGE_FILTERS._6m_to_9m));
        assert!(OVER_AGE_FILTERS._9m.includes(&AGE_RANGE_FILTERS._9m_to_1y));
        assert!(
            !OVER_AGE_FILTERS
                ._18m
                .includes(&AGE_RANGE_FILTERS._1y_to_18m)
        );
        assert!(
            OVER_AGE_FILTERS
                ._18m
                .includes(&AGE_RANGE_FILTERS._18m_to_2y)
        );
    }
}
