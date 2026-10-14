use std::result::Result as StdResult;

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

#[derive(Clone)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct DistributionStats<A> {
    /// Minimum value in the represented distribution.
    pub min: A,
    /// Maximum value in the represented distribution.
    pub max: A,
    /// 10th percentile of the represented distribution.
    pub pct10: A,
    /// 25th percentile of the represented distribution.
    pub pct25: A,
    /// Median of the represented distribution.
    pub median: A,
    /// 75th percentile of the represented distribution.
    pub pct75: A,
    /// 90th percentile of the represented distribution.
    pub pct90: A,
}

impl<A> DistributionStats<A> {
    const SUFFIXES: [&'static str; 7] =
        ["min", "max", "pct10", "pct25", "median", "pct75", "pct90"];

    pub fn as_mut_array(&mut self) -> [&mut A; 7] {
        [
            &mut self.min,
            &mut self.max,
            &mut self.pct10,
            &mut self.pct25,
            &mut self.median,
            &mut self.pct75,
            &mut self.pct90,
        ]
    }

    pub fn try_from_fn<E>(mut f: impl FnMut(&str) -> StdResult<A, E>) -> StdResult<Self, E> {
        Ok(Self {
            min: f(Self::SUFFIXES[0])?,
            max: f(Self::SUFFIXES[1])?,
            pct10: f(Self::SUFFIXES[2])?,
            pct25: f(Self::SUFFIXES[3])?,
            median: f(Self::SUFFIXES[4])?,
            pct75: f(Self::SUFFIXES[5])?,
            pct90: f(Self::SUFFIXES[6])?,
        })
    }
}
