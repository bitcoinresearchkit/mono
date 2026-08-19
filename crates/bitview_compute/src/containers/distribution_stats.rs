use brk_error::Result;

use std::result::Result as StdResult;

use bitview_traversable::Traversable;

#[derive(Clone, Traversable)]
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
    pub const SUFFIXES: [&'static str; 7] =
        ["min", "max", "pct10", "pct25", "median", "pct75", "pct90"];

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

    /// Apply a fallible operation to each of the 7 fields.
    pub fn try_for_each_mut(&mut self, mut f: impl FnMut(&mut A) -> Result<()>) -> Result<()> {
        f(&mut self.min)?;
        f(&mut self.max)?;
        f(&mut self.pct10)?;
        f(&mut self.pct25)?;
        f(&mut self.median)?;
        f(&mut self.pct75)?;
        f(&mut self.pct90)?;
        Ok(())
    }

    /// Get minimum value by applying a function to each field.
    pub fn min_by(&self, mut f: impl FnMut(&A) -> usize) -> usize {
        f(&self.min)
            .min(f(&self.max))
            .min(f(&self.pct10))
            .min(f(&self.pct25))
            .min(f(&self.median))
            .min(f(&self.pct75))
            .min(f(&self.pct90))
    }
}
