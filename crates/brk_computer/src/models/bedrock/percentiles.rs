use std::{fmt, str};

use brk_traversable::Traversable;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::Formattable;

use super::LossPercentileId;

#[derive(Debug, Clone, Copy, PartialEq, Traversable, Serialize, JsonSchema)]
pub struct Percentiles<T> {
    pub pct95: T,
    pub pct98: T,
    pub pct99: T,
    pub pct99_5: T,
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
    pub(super) fn from_fn(mut create: impl FnMut(LossPercentileId) -> T) -> Self {
        Self {
            pct95: create(LossPercentileId::Pct95),
            pct98: create(LossPercentileId::Pct98),
            pct99: create(LossPercentileId::Pct99),
            pct99_5: create(LossPercentileId::Pct99_5),
            pct99_9: create(LossPercentileId::Pct99_9),
        }
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &T> {
        [
            &self.pct95,
            &self.pct98,
            &self.pct99,
            &self.pct99_5,
            &self.pct99_9,
        ]
        .into_iter()
    }

    pub(super) fn map<U>(self, mut map: impl FnMut(T) -> U) -> Percentiles<U> {
        Percentiles {
            pct95: map(self.pct95),
            pct98: map(self.pct98),
            pct99: map(self.pct99),
            pct99_5: map(self.pct99_5),
            pct99_9: map(self.pct99_9),
        }
    }
}
