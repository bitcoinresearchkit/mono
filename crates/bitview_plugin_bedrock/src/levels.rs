use std::{fmt, str};

use bitview_traversable::Traversable;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::Formattable;

use super::LevelId;

#[derive(Debug, Clone, Copy, PartialEq, Traversable, Serialize, JsonSchema)]
pub struct Levels<T> {
    pub pct10: T,
    pub pct20: T,
    pub pct30: T,
    pub pct40: T,
    pub pct50: T,
    pub pct60: T,
    pub pct70: T,
    pub pct80: T,
    pub pct90: T,
}

impl_named_row_formattable!(Levels {
    pct10,
    pct20,
    pct30,
    pct40,
    pct50,
    pct60,
    pct70,
    pct80,
    pct90,
});

impl<T> Levels<T> {
    pub fn from_fn(mut create: impl FnMut(LevelId) -> T) -> Self {
        Self {
            pct10: create(LevelId::Pct10),
            pct20: create(LevelId::Pct20),
            pct30: create(LevelId::Pct30),
            pct40: create(LevelId::Pct40),
            pct50: create(LevelId::Pct50),
            pct60: create(LevelId::Pct60),
            pct70: create(LevelId::Pct70),
            pct80: create(LevelId::Pct80),
            pct90: create(LevelId::Pct90),
        }
    }

    pub fn map<U>(self, mut map: impl FnMut(T) -> U) -> Levels<U> {
        Levels {
            pct10: map(self.pct10),
            pct20: map(self.pct20),
            pct30: map(self.pct30),
            pct40: map(self.pct40),
            pct50: map(self.pct50),
            pct60: map(self.pct60),
            pct70: map(self.pct70),
            pct80: map(self.pct80),
            pct90: map(self.pct90),
        }
    }
}
