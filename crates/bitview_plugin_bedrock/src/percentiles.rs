use bitview_traversable::Traversable;

use super::LossPercentileId;

#[derive(Debug, Clone, Copy, PartialEq, Traversable)]
pub struct Percentiles<T> {
    /// Uses the 95th percentile of the mode's supply-in-loss history.
    pub pct95: T,
    /// Uses the 98th percentile of the mode's supply-in-loss history.
    pub pct98: T,
    /// Uses the 99th percentile of the mode's supply-in-loss history.
    pub pct99: T,
    /// Uses the 99.5th percentile of the mode's supply-in-loss history.
    pub pct99_5: T,
    /// Uses the 99.9th percentile of the mode's supply-in-loss history.
    pub pct99_9: T,
}

impl<T> Percentiles<T> {
    pub fn try_from_fn<E>(
        mut create: impl FnMut(LossPercentileId) -> Result<T, E>,
    ) -> Result<Self, E> {
        Ok(Self {
            pct95: create(LossPercentileId::Pct95)?,
            pct98: create(LossPercentileId::Pct98)?,
            pct99: create(LossPercentileId::Pct99)?,
            pct99_5: create(LossPercentileId::Pct99_5)?,
            pct99_9: create(LossPercentileId::Pct99_9)?,
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [
            &mut self.pct95,
            &mut self.pct98,
            &mut self.pct99,
            &mut self.pct99_5,
            &mut self.pct99_9,
        ]
        .into_iter()
    }
    pub fn from_fn(mut create: impl FnMut(LossPercentileId) -> T) -> Self {
        Self {
            pct95: create(LossPercentileId::Pct95),
            pct98: create(LossPercentileId::Pct98),
            pct99: create(LossPercentileId::Pct99),
            pct99_5: create(LossPercentileId::Pct99_5),
            pct99_9: create(LossPercentileId::Pct99_9),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [
            &self.pct95,
            &self.pct98,
            &self.pct99,
            &self.pct99_5,
            &self.pct99_9,
        ]
        .into_iter()
    }
}
