use std::result::Result as StdResult;

use bitview_cohort::AgeRangeId;
use bitview_traversable::Traversable;

#[derive(Clone, Copy, Debug, Default, PartialEq, Traversable)]
pub struct AgeCutoffs<T> {
    /// UTXOs younger than 120 days.
    pub under_4m: T,
    /// UTXOs younger than 150 days (STH).
    pub under_5m: T,
    /// UTXOs younger than 180 days.
    pub under_6m: T,
}

impl<T> AgeCutoffs<T> {
    pub fn from_fn(mut create: impl FnMut() -> T) -> Self {
        Self {
            under_4m: create(),
            under_5m: create(),
            under_6m: create(),
        }
    }

    pub fn containing_mut(&mut self, age: AgeRangeId) -> impl Iterator<Item = &mut T> {
        self.iter_mut()
            .zip([
                AgeRangeId::From4MTo5M,
                AgeRangeId::From5MTo6M,
                AgeRangeId::From6MTo9M,
            ])
            .filter_map(move |(value, excluded)| (age.index() < excluded.index()).then_some(value))
    }

    pub fn try_from_fn<E>(mut create: impl FnMut(&str) -> StdResult<T, E>) -> StdResult<Self, E> {
        Ok(Self {
            under_4m: create("under_4m")?,
            under_5m: create("under_5m")?,
            under_6m: create("under_6m")?,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.under_4m, &self.under_5m, &self.under_6m].into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.under_4m, &mut self.under_5m, &mut self.under_6m].into_iter()
    }
}
