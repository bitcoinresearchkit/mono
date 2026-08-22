use std::{fmt, str};

use bitview_traversable::Traversable;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::Formattable;

use super::LossPercentileId;

#[derive(Debug, Clone, Copy, PartialEq, Traversable, Serialize, JsonSchema)]
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

impl_named_row_formattable!(Percentiles {
    pct95,
    pct98,
    pct99,
    pct99_5,
    pct99_9,
});

impl<T> Percentiles<T> {
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

    pub fn map<U>(self, mut map: impl FnMut(T) -> U) -> Percentiles<U> {
        Percentiles {
            pct95: map(self.pct95),
            pct98: map(self.pct98),
            pct99: map(self.pct99),
            pct99_5: map(self.pct99_5),
            pct99_9: map(self.pct99_9),
        }
    }
}
