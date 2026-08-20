use std::ops::Range;

use brk_types::Sats;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AmountFilter {
    LowerThan(Sats),
    Range(Range<Sats>),
    GreaterOrEqual(Sats),
}

impl AmountFilter {
    pub fn contains(&self, sats: Sats) -> bool {
        match self {
            AmountFilter::LowerThan(max) => sats < *max,
            AmountFilter::Range(r) => sats >= r.start && sats < r.end,
            AmountFilter::GreaterOrEqual(min) => sats >= *min,
        }
    }

    pub fn includes(&self, other: &AmountFilter) -> bool {
        match self {
            AmountFilter::LowerThan(max) => match other {
                AmountFilter::LowerThan(max2) => max >= max2,
                AmountFilter::Range(range) => range.end <= *max,
                AmountFilter::GreaterOrEqual(_) => false,
            },
            AmountFilter::GreaterOrEqual(min) => match other {
                AmountFilter::Range(range) => range.start >= *min,
                AmountFilter::GreaterOrEqual(min2) => min <= min2,
                AmountFilter::LowerThan(_) => false,
            },
            AmountFilter::Range(_) => false,
        }
    }
}
