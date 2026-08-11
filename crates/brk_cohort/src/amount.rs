use brk_traversable::Traversable;
use rayon::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::ColumnId;

use crate::{
    AmountRange, AmountRangeId, Filter, OVER_AMOUNT_FILTERS, OverAmount, OverAmountId,
    UNDER_AMOUNT_FILTERS, UnderAmount, UnderAmountId,
};

#[derive(Debug, Default, Clone, Traversable, Serialize, JsonSchema)]
pub struct Amount<T> {
    pub range: AmountRange<T>,
    pub under: UnderAmount<T>,
    pub over: OverAmount<T>,
}

impl<T> Amount<T> {
    pub fn new(mut create: impl FnMut(Filter, &'static str) -> T) -> Self {
        Self {
            range: AmountRange::new(&mut create),
            under: UnderAmount::new(&mut create),
            over: OverAmount::new(create),
        }
    }

    pub fn try_new<E>(
        mut create: impl FnMut(Filter, &'static str) -> Result<T, E>,
    ) -> Result<Self, E> {
        Ok(Self {
            range: AmountRange::try_new(&mut create)?,
            under: UnderAmount::try_new(&mut create)?,
            over: OverAmount::try_new(create)?,
        })
    }

    pub fn get(&self, filter: &Filter) -> Option<&T> {
        AmountRangeId::matching(filter)
            .map(|id| id.select(&self.range))
            .or_else(|| {
                UnderAmountId::ALL
                    .iter()
                    .copied()
                    .find(|id| id.select(&UNDER_AMOUNT_FILTERS) == filter)
                    .map(|id| id.select(&self.under))
            })
            .or_else(|| {
                OverAmountId::ALL
                    .iter()
                    .copied()
                    .find(|id| id.select(&OVER_AMOUNT_FILTERS) == filter)
                    .map(|id| id.select(&self.over))
            })
    }

    pub fn map_named<U>(&self, mut map: impl FnMut(&Filter, &'static str, &T) -> U) -> Amount<U> {
        Amount {
            range: AmountRange::new(|filter, name| {
                map(
                    &filter,
                    name,
                    self.get(&filter).expect("exact amount range"),
                )
            }),
            under: UnderAmount::new(|filter, name| {
                map(
                    &filter,
                    name,
                    self.get(&filter).expect("under-amount threshold"),
                )
            }),
            over: OverAmount::new(|filter, name| {
                map(
                    &filter,
                    name,
                    self.get(&filter).expect("over-amount threshold"),
                )
            }),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.range
            .iter()
            .chain(self.under.iter())
            .chain(self.over.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.range
            .iter_mut()
            .chain(self.under.iter_mut())
            .chain(self.over.iter_mut())
    }

    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        self.range
            .par_iter_mut()
            .chain(self.under.par_iter_mut())
            .chain(self.over.par_iter_mut())
    }
}
