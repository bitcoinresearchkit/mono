use std::result::Result as StdResult;

use bitview_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct DistributionStats<A> {
    /// Minimum value in the represented distribution.
    pub min: A,
    /// Maximum value in the represented distribution.
    pub max: A,
    /// 10th percentile of the represented distribution.
    pub p10: A,
    /// 25th percentile of the represented distribution.
    pub p25: A,
    /// Median of the represented distribution.
    pub median: A,
    /// 75th percentile of the represented distribution.
    pub p75: A,
    /// 90th percentile of the represented distribution.
    pub p90: A,
}

impl<A> DistributionStats<A> {
    const SUFFIXES: [&'static str; 7] =
        ["min", "max", "p10", "p25", "median", "p75", "p90"];

    pub fn as_mut_array(&mut self) -> [&mut A; 7] {
        [
            &mut self.min,
            &mut self.max,
            &mut self.p10,
            &mut self.p25,
            &mut self.median,
            &mut self.p75,
            &mut self.p90,
        ]
    }

    /// Each statistic mapped with its suffix (`min`, `median`, ...).
    pub fn map_with_suffix<B>(&self, mut f: impl FnMut(&str, &A) -> B) -> DistributionStats<B> {
        DistributionStats {
            min: f(Self::SUFFIXES[0], &self.min),
            max: f(Self::SUFFIXES[1], &self.max),
            p10: f(Self::SUFFIXES[2], &self.p10),
            p25: f(Self::SUFFIXES[3], &self.p25),
            median: f(Self::SUFFIXES[4], &self.median),
            p75: f(Self::SUFFIXES[5], &self.p75),
            p90: f(Self::SUFFIXES[6], &self.p90),
        }
    }

    pub fn try_from_fn<E>(mut f: impl FnMut(&str) -> StdResult<A, E>) -> StdResult<Self, E> {
        Ok(Self {
            min: f(Self::SUFFIXES[0])?,
            max: f(Self::SUFFIXES[1])?,
            p10: f(Self::SUFFIXES[2])?,
            p25: f(Self::SUFFIXES[3])?,
            median: f(Self::SUFFIXES[4])?,
            p75: f(Self::SUFFIXES[5])?,
            p90: f(Self::SUFFIXES[6])?,
        })
    }
}
