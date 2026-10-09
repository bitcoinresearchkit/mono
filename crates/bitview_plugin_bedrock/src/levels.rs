use std::{fmt, str};

use bitview_traversable::Traversable;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::Formattable;

use super::LevelId;

#[derive(Debug, Clone, Copy, PartialEq, Traversable, Serialize, JsonSchema)]
pub struct Levels<T> {
    /// Returns the 10th percentile of that subset's creation-price distribution.
    pub pct10: T,
    /// Returns the 20th percentile of that subset's creation-price distribution.
    pub pct20: T,
    /// Returns the 30th percentile of that subset's creation-price distribution.
    pub pct30: T,
    /// Returns the 40th percentile of that subset's creation-price distribution.
    pub pct40: T,
    /// Returns the 50th percentile of that subset's creation-price distribution.
    pub median: T,
    /// Returns the 60th percentile of that subset's creation-price distribution.
    pub pct60: T,
    /// Returns the 70th percentile of that subset's creation-price distribution.
    pub pct70: T,
    /// Returns the 80th percentile of that subset's creation-price distribution.
    pub pct80: T,
    /// Returns the 90th percentile of that subset's creation-price distribution.
    pub pct90: T,
}

impl_named_row_formattable!(Levels {
    pct10,
    pct20,
    pct30,
    pct40,
    median,
    pct60,
    pct70,
    pct80,
    pct90,
});

impl<T> Levels<T> {
    pub fn try_from_fn<E>(mut create: impl FnMut(LevelId) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            pct10: create(LevelId::Pct10)?,
            pct20: create(LevelId::Pct20)?,
            pct30: create(LevelId::Pct30)?,
            pct40: create(LevelId::Pct40)?,
            median: create(LevelId::Median)?,
            pct60: create(LevelId::Pct60)?,
            pct70: create(LevelId::Pct70)?,
            pct80: create(LevelId::Pct80)?,
            pct90: create(LevelId::Pct90)?,
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [
            &mut self.pct10,
            &mut self.pct20,
            &mut self.pct30,
            &mut self.pct40,
            &mut self.median,
            &mut self.pct60,
            &mut self.pct70,
            &mut self.pct80,
            &mut self.pct90,
        ]
        .into_iter()
    }
    pub fn from_fn(mut create: impl FnMut(LevelId) -> T) -> Self {
        Self {
            pct10: create(LevelId::Pct10),
            pct20: create(LevelId::Pct20),
            pct30: create(LevelId::Pct30),
            pct40: create(LevelId::Pct40),
            median: create(LevelId::Median),
            pct60: create(LevelId::Pct60),
            pct70: create(LevelId::Pct70),
            pct80: create(LevelId::Pct80),
            pct90: create(LevelId::Pct90),
        }
    }
}
