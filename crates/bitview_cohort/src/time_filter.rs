use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeFilter {
    LowerThan(usize),
    Range(Range<usize>),
    GreaterOrEqual(usize),
}

impl TimeFilter {
    pub fn contains(&self, hours: usize) -> bool {
        match self {
            TimeFilter::LowerThan(max) => hours < *max,
            TimeFilter::Range(r) => r.contains(&hours),
            TimeFilter::GreaterOrEqual(min) => hours >= *min,
        }
    }

    pub fn includes(&self, other: &TimeFilter) -> bool {
        match self {
            TimeFilter::LowerThan(max) => match other {
                TimeFilter::LowerThan(max2) => max >= max2,
                TimeFilter::Range(range) => range.end <= *max,
                TimeFilter::GreaterOrEqual(_) => false,
            },
            TimeFilter::GreaterOrEqual(min) => match other {
                TimeFilter::Range(range) => range.start >= *min,
                TimeFilter::GreaterOrEqual(min2) => min <= min2,
                TimeFilter::LowerThan(_) => false,
            },
            TimeFilter::Range(_) => false,
        }
    }

    /// Returns true if this filter includes day 0 (UTXOs less than 1 day old)
    pub fn includes_first_day(&self) -> bool {
        match self {
            TimeFilter::LowerThan(_) => true,
            TimeFilter::Range(r) => r.start == 0,
            TimeFilter::GreaterOrEqual(_) => false,
        }
    }
}
