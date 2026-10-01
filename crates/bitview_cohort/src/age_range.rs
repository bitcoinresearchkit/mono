use std::ops::Range;

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use brk_types::Age;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{CohortContext, CohortId, CohortName, Term};

// Age boundary constants in hours
const HOURS_1H: usize = 1;
const HOURS_1D: usize = 24;
const HOURS_1W: usize = 24 * 7;
const HOURS_1M: usize = 24 * 30;
const HOURS_2M: usize = 24 * 2 * 30;
const HOURS_3M: usize = 24 * 3 * 30;
pub const HOURS_4M: usize = 24 * 4 * 30;
pub const HOURS_5M: usize = Term::THRESHOLD_HOURS;
pub const HOURS_6M: usize = 24 * 6 * 30;
const HOURS_9M: usize = HOURS_6M + HOURS_3M;
const HOURS_1Y: usize = 24 * 365;
// Keep year-based ranges anchored to the existing 365-day year convention.
const HOURS_18M: usize = HOURS_1Y + HOURS_6M;
const HOURS_2Y: usize = 24 * 2 * 365;
const HOURS_3Y: usize = 24 * 3 * 365;
const HOURS_4Y: usize = 24 * 4 * 365;
const HOURS_5Y: usize = 24 * 5 * 365;
const HOURS_6Y: usize = 24 * 6 * 365;
const HOURS_7Y: usize = 24 * 7 * 365;
const HOURS_8Y: usize = 24 * 8 * 365;
const HOURS_10Y: usize = 24 * 10 * 365;
const HOURS_12Y: usize = 24 * 12 * 365;
const HOURS_15Y: usize = 24 * 15 * 365;

pub const AGE_RANGE_COUNT: usize = 23;
const STH_AGE_RANGE_COUNT: usize = AgeRangeId::From5MTo6M.index();
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum AgeRangeId {
    Under1H,
    From1HTo1D,
    From1DTo1W,
    From1WTo1M,
    From1MTo2M,
    From2MTo3M,
    From3MTo4M,
    From4MTo5M,
    From5MTo6M,
    From6MTo9M,
    From9MTo1Y,
    From1YTo18M,
    From18MTo2Y,
    From2YTo3Y,
    From3YTo4Y,
    From4YTo5Y,
    From5YTo6Y,
    From6YTo7Y,
    From7YTo8Y,
    From8YTo10Y,
    From10YTo12Y,
    From12YTo15Y,
    Over15Y,
}

pub const AGE_RANGE_IDS: [AgeRangeId; AGE_RANGE_COUNT] = [
    AgeRangeId::Under1H,
    AgeRangeId::From1HTo1D,
    AgeRangeId::From1DTo1W,
    AgeRangeId::From1WTo1M,
    AgeRangeId::From1MTo2M,
    AgeRangeId::From2MTo3M,
    AgeRangeId::From3MTo4M,
    AgeRangeId::From4MTo5M,
    AgeRangeId::From5MTo6M,
    AgeRangeId::From6MTo9M,
    AgeRangeId::From9MTo1Y,
    AgeRangeId::From1YTo18M,
    AgeRangeId::From18MTo2Y,
    AgeRangeId::From2YTo3Y,
    AgeRangeId::From3YTo4Y,
    AgeRangeId::From4YTo5Y,
    AgeRangeId::From5YTo6Y,
    AgeRangeId::From6YTo7Y,
    AgeRangeId::From7YTo8Y,
    AgeRangeId::From8YTo10Y,
    AgeRangeId::From10YTo12Y,
    AgeRangeId::From12YTo15Y,
    AgeRangeId::Over15Y,
];

pub const STH_AGE_RANGE_IDS: &[AgeRangeId] = AGE_RANGE_IDS.split_at(STH_AGE_RANGE_COUNT).0;
pub const LTH_AGE_RANGE_IDS: &[AgeRangeId] = AGE_RANGE_IDS.split_at(STH_AGE_RANGE_COUNT).1;

impl AgeRangeId {
    pub const ALL: &'static [Self] = &AGE_RANGE_IDS;

    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn cohort(self) -> CohortId {
        CohortId::Age(self)
    }

    pub fn from_cohort_name(context: CohortContext, name: &str) -> Option<Self> {
        let name = name.strip_prefix(context.prefix())?.strip_prefix('_')?;
        Self::ALL.iter().copied().find(|id| id.name().id == name)
    }

    #[inline]
    pub fn bounds(self) -> &'static Range<usize> {
        self.select(&AGE_RANGE_BOUNDS)
    }

    #[inline]
    pub fn name(self) -> &'static CohortName {
        self.select(&AGE_RANGE_NAMES)
    }

    pub fn term(self) -> Term {
        if self.bounds().end <= Term::THRESHOLD_HOURS {
            Term::Sth
        } else {
            Term::Lth
        }
    }

    pub fn select<T>(self, values: &AgeRange<T>) -> &T {
        match self {
            Self::Under1H => &values.under_1h,
            Self::From1HTo1D => &values._1h_to_1d,
            Self::From1DTo1W => &values._1d_to_1w,
            Self::From1WTo1M => &values._1w_to_1m,
            Self::From1MTo2M => &values._1m_to_2m,
            Self::From2MTo3M => &values._2m_to_3m,
            Self::From3MTo4M => &values._3m_to_4m,
            Self::From4MTo5M => &values._4m_to_5m,
            Self::From5MTo6M => &values._5m_to_6m,
            Self::From6MTo9M => &values._6m_to_9m,
            Self::From9MTo1Y => &values._9m_to_1y,
            Self::From1YTo18M => &values._1y_to_18m,
            Self::From18MTo2Y => &values._18m_to_2y,
            Self::From2YTo3Y => &values._2y_to_3y,
            Self::From3YTo4Y => &values._3y_to_4y,
            Self::From4YTo5Y => &values._4y_to_5y,
            Self::From5YTo6Y => &values._5y_to_6y,
            Self::From6YTo7Y => &values._6y_to_7y,
            Self::From7YTo8Y => &values._7y_to_8y,
            Self::From8YTo10Y => &values._8y_to_10y,
            Self::From10YTo12Y => &values._10y_to_12y,
            Self::From12YTo15Y => &values._12y_to_15y,
            Self::Over15Y => &values.over_15y,
        }
    }

    pub fn select_mut<T>(self, values: &mut AgeRange<T>) -> &mut T {
        match self {
            Self::Under1H => &mut values.under_1h,
            Self::From1HTo1D => &mut values._1h_to_1d,
            Self::From1DTo1W => &mut values._1d_to_1w,
            Self::From1WTo1M => &mut values._1w_to_1m,
            Self::From1MTo2M => &mut values._1m_to_2m,
            Self::From2MTo3M => &mut values._2m_to_3m,
            Self::From3MTo4M => &mut values._3m_to_4m,
            Self::From4MTo5M => &mut values._4m_to_5m,
            Self::From5MTo6M => &mut values._5m_to_6m,
            Self::From6MTo9M => &mut values._6m_to_9m,
            Self::From9MTo1Y => &mut values._9m_to_1y,
            Self::From1YTo18M => &mut values._1y_to_18m,
            Self::From18MTo2Y => &mut values._18m_to_2y,
            Self::From2YTo3Y => &mut values._2y_to_3y,
            Self::From3YTo4Y => &mut values._3y_to_4y,
            Self::From4YTo5Y => &mut values._4y_to_5y,
            Self::From5YTo6Y => &mut values._5y_to_6y,
            Self::From6YTo7Y => &mut values._6y_to_7y,
            Self::From7YTo8Y => &mut values._7y_to_8y,
            Self::From8YTo10Y => &mut values._8y_to_10y,
            Self::From10YTo12Y => &mut values._10y_to_12y,
            Self::From12YTo15Y => &mut values._12y_to_15y,
            Self::Over15Y => &mut values.over_15y,
        }
    }

    pub fn series<T>(
        context: CohortContext,
        mut create: impl FnMut(Self, &str) -> T,
    ) -> AgeRange<T> {
        AgeRange::from_fn(|id| {
            let name = context.prefixed(id.name().id);
            create(id, &name)
        })
    }
}

impl From<Age> for AgeRangeId {
    #[inline]
    fn from(age: Age) -> Self {
        match age.hours() {
            0..HOURS_1H => Self::Under1H,
            HOURS_1H..HOURS_1D => Self::From1HTo1D,
            HOURS_1D..HOURS_1W => Self::From1DTo1W,
            HOURS_1W..HOURS_1M => Self::From1WTo1M,
            HOURS_1M..HOURS_2M => Self::From1MTo2M,
            HOURS_2M..HOURS_3M => Self::From2MTo3M,
            HOURS_3M..HOURS_4M => Self::From3MTo4M,
            HOURS_4M..HOURS_5M => Self::From4MTo5M,
            HOURS_5M..HOURS_6M => Self::From5MTo6M,
            HOURS_6M..HOURS_9M => Self::From6MTo9M,
            HOURS_9M..HOURS_1Y => Self::From9MTo1Y,
            HOURS_1Y..HOURS_18M => Self::From1YTo18M,
            HOURS_18M..HOURS_2Y => Self::From18MTo2Y,
            HOURS_2Y..HOURS_3Y => Self::From2YTo3Y,
            HOURS_3Y..HOURS_4Y => Self::From3YTo4Y,
            HOURS_4Y..HOURS_5Y => Self::From4YTo5Y,
            HOURS_5Y..HOURS_6Y => Self::From5YTo6Y,
            HOURS_6Y..HOURS_7Y => Self::From6YTo7Y,
            HOURS_7Y..HOURS_8Y => Self::From7YTo8Y,
            HOURS_8Y..HOURS_10Y => Self::From8YTo10Y,
            HOURS_10Y..HOURS_12Y => Self::From10YTo12Y,
            HOURS_12Y..HOURS_15Y => Self::From12YTo15Y,
            _ => Self::Over15Y,
        }
    }
}

/// Age boundaries in hours. Defines the cohort ranges:
/// [0, 1h), [1h, 1d), [1d, 1w), [1w, 1m), ..., [15y, ∞)
pub const AGE_BOUNDARIES: [usize; AGE_RANGE_COUNT - 1] = [
    HOURS_1H, HOURS_1D, HOURS_1W, HOURS_1M, HOURS_2M, HOURS_3M, HOURS_4M, HOURS_5M, HOURS_6M,
    HOURS_9M, HOURS_1Y, HOURS_18M, HOURS_2Y, HOURS_3Y, HOURS_4Y, HOURS_5Y, HOURS_6Y, HOURS_7Y,
    HOURS_8Y, HOURS_10Y, HOURS_12Y, HOURS_15Y,
];

/// Age range bounds (end = usize::MAX means unbounded)
const AGE_RANGE_BOUNDS: AgeRange<Range<usize>> = AgeRange {
    under_1h: 0..HOURS_1H,
    _1h_to_1d: HOURS_1H..HOURS_1D,
    _1d_to_1w: HOURS_1D..HOURS_1W,
    _1w_to_1m: HOURS_1W..HOURS_1M,
    _1m_to_2m: HOURS_1M..HOURS_2M,
    _2m_to_3m: HOURS_2M..HOURS_3M,
    _3m_to_4m: HOURS_3M..HOURS_4M,
    _4m_to_5m: HOURS_4M..HOURS_5M,
    _5m_to_6m: HOURS_5M..HOURS_6M,
    _6m_to_9m: HOURS_6M..HOURS_9M,
    _9m_to_1y: HOURS_9M..HOURS_1Y,
    _1y_to_18m: HOURS_1Y..HOURS_18M,
    _18m_to_2y: HOURS_18M..HOURS_2Y,
    _2y_to_3y: HOURS_2Y..HOURS_3Y,
    _3y_to_4y: HOURS_3Y..HOURS_4Y,
    _4y_to_5y: HOURS_4Y..HOURS_5Y,
    _5y_to_6y: HOURS_5Y..HOURS_6Y,
    _6y_to_7y: HOURS_6Y..HOURS_7Y,
    _7y_to_8y: HOURS_7Y..HOURS_8Y,
    _8y_to_10y: HOURS_8Y..HOURS_10Y,
    _10y_to_12y: HOURS_10Y..HOURS_12Y,
    _12y_to_15y: HOURS_12Y..HOURS_15Y,
    over_15y: HOURS_15Y..usize::MAX,
};

/// Age range names
pub const AGE_RANGE_NAMES: AgeRange<CohortName> = AgeRange {
    under_1h: CohortName::new("under_1h_old", "<1h", "Under 1 Hour Old"),
    _1h_to_1d: CohortName::new("1h_to_1d_old", "1h-1d", "1 Hour to 1 Day Old"),
    _1d_to_1w: CohortName::new("1d_to_1w_old", "1d-1w", "1 Day to 1 Week Old"),
    _1w_to_1m: CohortName::new("1w_to_1m_old", "1w-1m", "1 Week to 1 Month Old"),
    _1m_to_2m: CohortName::new("1m_to_2m_old", "1m-2m", "1 to 2 Months Old"),
    _2m_to_3m: CohortName::new("2m_to_3m_old", "2m-3m", "2 to 3 Months Old"),
    _3m_to_4m: CohortName::new("3m_to_4m_old", "3m-4m", "3 to 4 Months Old"),
    _4m_to_5m: CohortName::new("4m_to_5m_old", "4m-5m", "4 to 5 Months Old"),
    _5m_to_6m: CohortName::new("5m_to_6m_old", "5m-6m", "5 to 6 Months Old"),
    _6m_to_9m: CohortName::new("6m_to_9m_old", "6m-9m", "6 to 9 Months Old"),
    _9m_to_1y: CohortName::new("9m_to_1y_old", "9m-1y", "9 Months to 1 Year Old"),
    _1y_to_18m: CohortName::new("1y_to_18m_old", "1y-18m", "1 Year to 18 Months Old"),
    _18m_to_2y: CohortName::new("18m_to_2y_old", "18m-2y", "18 Months to 2 Years Old"),
    _2y_to_3y: CohortName::new("2y_to_3y_old", "2y-3y", "2 to 3 Years Old"),
    _3y_to_4y: CohortName::new("3y_to_4y_old", "3y-4y", "3 to 4 Years Old"),
    _4y_to_5y: CohortName::new("4y_to_5y_old", "4y-5y", "4 to 5 Years Old"),
    _5y_to_6y: CohortName::new("5y_to_6y_old", "5y-6y", "5 to 6 Years Old"),
    _6y_to_7y: CohortName::new("6y_to_7y_old", "6y-7y", "6 to 7 Years Old"),
    _7y_to_8y: CohortName::new("7y_to_8y_old", "7y-8y", "7 to 8 Years Old"),
    _8y_to_10y: CohortName::new("8y_to_10y_old", "8y-10y", "8 to 10 Years Old"),
    _10y_to_12y: CohortName::new("10y_to_12y_old", "10y-12y", "10 to 12 Years Old"),
    _12y_to_15y: CohortName::new("12y_to_15y_old", "12y-15y", "12 to 15 Years Old"),
    over_15y: CohortName::new("over_15y_old", "15y+", "15+ Years Old"),
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct AgeRange<T> {
    /// Uses UTXOs less than 1 hour old.
    pub under_1h: T,
    /// Uses UTXOs at least 1 hour and less than 1 day old.
    pub _1h_to_1d: T,
    /// Uses UTXOs at least 1 day and less than 7 days old.
    pub _1d_to_1w: T,
    /// Uses UTXOs at least 7 days and less than 30 days old.
    pub _1w_to_1m: T,
    /// Uses UTXOs at least 30 days and less than 60 days old.
    pub _1m_to_2m: T,
    /// Uses UTXOs at least 60 days and less than 90 days old.
    pub _2m_to_3m: T,
    /// Uses UTXOs at least 90 days and less than 120 days old.
    pub _3m_to_4m: T,
    /// Uses UTXOs at least 120 days and less than 150 days old.
    pub _4m_to_5m: T,
    /// Uses UTXOs at least 150 days and less than 180 days old.
    pub _5m_to_6m: T,
    /// Uses UTXOs at least 180 days and less than 270 days old.
    pub _6m_to_9m: T,
    /// Uses UTXOs at least 270 days and less than 365 days old.
    pub _9m_to_1y: T,
    /// Uses UTXOs at least 365 days and less than 545 days old.
    pub _1y_to_18m: T,
    /// Uses UTXOs at least 545 days and less than 730 days old.
    pub _18m_to_2y: T,
    /// Uses UTXOs at least 2 years and less than 3 years old, using 365-day
    /// years.
    pub _2y_to_3y: T,
    /// Uses UTXOs at least 3 years and less than 4 years old, using 365-day
    /// years.
    pub _3y_to_4y: T,
    /// Uses UTXOs at least 4 years and less than 5 years old, using 365-day
    /// years.
    pub _4y_to_5y: T,
    /// Uses UTXOs at least 5 years and less than 6 years old, using 365-day
    /// years.
    pub _5y_to_6y: T,
    /// Uses UTXOs at least 6 years and less than 7 years old, using 365-day
    /// years.
    pub _6y_to_7y: T,
    /// Uses UTXOs at least 7 years and less than 8 years old, using 365-day
    /// years.
    pub _7y_to_8y: T,
    /// Uses UTXOs at least 8 years and less than 10 years old, using 365-day
    /// years.
    pub _8y_to_10y: T,
    /// Uses UTXOs at least 10 years and less than 12 years old, using 365-day
    /// years.
    pub _10y_to_12y: T,
    /// Uses UTXOs at least 12 years and less than 15 years old, using 365-day
    /// years.
    pub _12y_to_15y: T,
    /// Uses UTXOs at least 15 years old, using 365-day years.
    pub over_15y: T,
}

impl_cohort_collection!(AgeRangeId for AgeRange {
    Under1H => under_1h,
    From1HTo1D => _1h_to_1d,
    From1DTo1W => _1d_to_1w,
    From1WTo1M => _1w_to_1m,
    From1MTo2M => _1m_to_2m,
    From2MTo3M => _2m_to_3m,
    From3MTo4M => _3m_to_4m,
    From4MTo5M => _4m_to_5m,
    From5MTo6M => _5m_to_6m,
    From6MTo9M => _6m_to_9m,
    From9MTo1Y => _9m_to_1y,
    From1YTo18M => _1y_to_18m,
    From18MTo2Y => _18m_to_2y,
    From2YTo3Y => _2y_to_3y,
    From3YTo4Y => _3y_to_4y,
    From4YTo5Y => _4y_to_5y,
    From5YTo6Y => _5y_to_6y,
    From6YTo7Y => _6y_to_7y,
    From7YTo8Y => _7y_to_8y,
    From8YTo10Y => _8y_to_10y,
    From10YTo12Y => _10y_to_12y,
    From12YTo15Y => _12y_to_15y,
    Over15Y => over_15y,
});

impl<T> AgeRange<T> {
    /// Get mutable reference by Age. O(1).
    #[inline]
    pub fn get_mut(&mut self, age: Age) -> &mut T {
        AgeRangeId::from(age).select_mut(self)
    }

    pub fn new(mut create: impl FnMut(CohortId) -> T) -> Self {
        Self::from_fn(|id| create(id.cohort()))
    }

    pub fn try_new<E>(mut create: impl FnMut(CohortId) -> Result<T, E>) -> Result<Self, E> {
        Self::try_from_fn(|id| create(id.cohort()))
    }
}

#[cfg(test)]
mod tests {
    use brk_types::Timestamp;

    use super::*;

    #[test]
    fn ranges_are_contiguous_and_match_the_declared_count() {
        let ranges: Vec<_> = AGE_RANGE_BOUNDS.iter().collect();

        assert_eq!(ranges.len(), AGE_RANGE_COUNT);
        assert_eq!(ranges.first().unwrap().start, 0);
        assert_eq!(ranges.last().unwrap().end, usize::MAX);

        for adjacent in ranges.windows(2) {
            assert_eq!(adjacent[0].end, adjacent[1].start);
        }
    }

    #[test]
    fn cohort_ids_match_storage_order_and_term_split() {
        assert_eq!(AgeRangeId::ALL, &AGE_RANGE_IDS);
        assert_eq!(
            STH_AGE_RANGE_IDS.len() + LTH_AGE_RANGE_IDS.len(),
            AGE_RANGE_COUNT
        );
        assert_eq!(STH_AGE_RANGE_IDS, &AGE_RANGE_IDS[..STH_AGE_RANGE_COUNT]);
        assert_eq!(LTH_AGE_RANGE_IDS, &AGE_RANGE_IDS[STH_AGE_RANGE_COUNT..]);
        assert_eq!(STH_AGE_RANGE_IDS.last(), Some(&AgeRangeId::From4MTo5M));
        assert_eq!(LTH_AGE_RANGE_IDS.first(), Some(&AgeRangeId::From5MTo6M));

        let values = AgeRange::from_fn(|id| id.index());
        for (index, &id) in AGE_RANGE_IDS.iter().enumerate() {
            assert_eq!(id.index(), index);
            assert_eq!(*id.select(&values), index);
        }
    }

    #[test]
    fn cohort_ids_select_named_fields_and_metadata() {
        let mut named = AgeRange::from_fn(|id| id.index());
        for &id in &AGE_RANGE_IDS {
            assert_eq!(*id.select(&named), id.index());
            *id.select_mut(&mut named) += AGE_RANGE_COUNT;
            assert_eq!(*id.select(&named), id.index() + AGE_RANGE_COUNT);
            assert_eq!(id.bounds(), id.select(&AGE_RANGE_BOUNDS));
            assert_eq!(id.cohort(), CohortId::Age(id));
            assert_eq!(id.cohort().name(), id.name().id);
            assert_eq!(
                id.term(),
                if id.index() < STH_AGE_RANGE_COUNT {
                    Term::Sth
                } else {
                    Term::Lth
                }
            );
            assert_eq!(id.name().id, id.select(&AGE_RANGE_NAMES).id);
        }

        let series = AgeRangeId::series(CohortContext::Utxo, |id, name| (id, name.to_owned()));
        for (((id, name), expected_id), expected_name) in
            series.iter().zip(AGE_RANGE_IDS).zip(AGE_RANGE_NAMES.iter())
        {
            assert_eq!(*id, expected_id);
            assert_eq!(name, &CohortContext::Utxo.prefixed(expected_name.id));
        }
    }

    #[test]
    fn cohort_names_roundtrip_to_typed_ids() {
        for context in [CohortContext::Utxo, CohortContext::Addr] {
            for id in AGE_RANGE_IDS {
                let name = context.prefixed(id.name().id);
                assert_eq!(AgeRangeId::from_cohort_name(context, &name), Some(id));
            }
        }
        assert_eq!(
            AgeRangeId::from_cohort_name(CohortContext::Utxo, "all"),
            None
        );
    }

    #[test]
    fn split_range_names_and_boundaries_match() {
        assert_eq!(HOURS_9M, HOURS_6M + HOURS_3M);
        assert_eq!(HOURS_18M, HOURS_1Y + HOURS_6M);
        assert_eq!(AGE_RANGE_NAMES._6m_to_9m.id, "6m_to_9m_old");
        assert_eq!(AGE_RANGE_NAMES._9m_to_1y.id, "9m_to_1y_old");
        assert_eq!(AGE_RANGE_NAMES._1y_to_18m.id, "1y_to_18m_old");
        assert_eq!(AGE_RANGE_NAMES._18m_to_2y.id, "18m_to_2y_old");
    }

    #[test]
    fn classifier_matches_every_typed_range_boundary() {
        for &id in &AGE_RANGE_IDS {
            let bounds = id.bounds();
            let at_start = Age::new(
                Timestamp::new((bounds.start * 60 * 60) as u32),
                Timestamp::ZERO,
            );
            assert_eq!(AgeRangeId::from(at_start), id);

            if bounds.end != usize::MAX {
                let before_end = Age::new(
                    Timestamp::new(((bounds.end - 1) * 60 * 60) as u32),
                    Timestamp::ZERO,
                );
                assert_eq!(AgeRangeId::from(before_end), id);
            }
        }
    }

    #[test]
    fn eighteen_month_classifier_stays_anchored_to_one_year_plus_six_months() {
        let age_at_540_days = Age::new(
            Timestamp::new((HOURS_6M * 3 * 60 * 60) as u32),
            Timestamp::ZERO,
        );
        let age_at_18m = Age::new(
            Timestamp::new((HOURS_18M * 60 * 60) as u32),
            Timestamp::ZERO,
        );

        assert_eq!(
            AgeRangeId::from(age_at_540_days)
                .select(&AGE_RANGE_NAMES)
                .id,
            "1y_to_18m_old"
        );
        assert_eq!(
            AgeRangeId::from(age_at_18m).select(&AGE_RANGE_NAMES).id,
            "18m_to_2y_old"
        );
    }
}
