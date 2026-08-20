use bitview_traversable::Traversable;
use rayon::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::ColumnId;

use crate::{
    AGE_RANGE_FILTERS, AgeRange, AgeRangeId, Filter, OVER_AGE_FILTERS, OverAge, OverAgeId,
    UNDER_AGE_FILTERS, UnderAge, UnderAgeId,
};

#[derive(Debug, Default, Clone, Traversable, Serialize, JsonSchema)]
pub struct ByAge<T> {
    pub range: AgeRange<T>,
    pub under: UnderAge<T>,
    pub over: OverAge<T>,
}

impl<T> ByAge<T> {
    pub fn new(mut create: impl FnMut(Filter, &'static str) -> T) -> Self {
        Self {
            range: AgeRange::new(&mut create),
            under: UnderAge::new(&mut create),
            over: OverAge::new(create),
        }
    }

    pub fn try_new<E>(
        mut create: impl FnMut(Filter, &'static str) -> Result<T, E>,
    ) -> Result<Self, E> {
        Ok(Self {
            range: AgeRange::try_new(&mut create)?,
            under: UnderAge::try_new(&mut create)?,
            over: OverAge::try_new(create)?,
        })
    }

    pub fn get(&self, filter: &Filter) -> Option<&T> {
        AgeRangeId::ALL
            .iter()
            .copied()
            .find(|id| id.select(&AGE_RANGE_FILTERS) == filter)
            .map(|id| id.select(&self.range))
            .or_else(|| {
                UnderAgeId::ALL
                    .iter()
                    .copied()
                    .find(|id| id.select(&UNDER_AGE_FILTERS) == filter)
                    .map(|id| id.select(&self.under))
            })
            .or_else(|| {
                OverAgeId::ALL
                    .iter()
                    .copied()
                    .find(|id| id.select(&OVER_AGE_FILTERS) == filter)
                    .map(|id| id.select(&self.over))
            })
    }

    pub fn map_named<U>(&self, mut map: impl FnMut(&Filter, &'static str, &T) -> U) -> ByAge<U> {
        ByAge {
            range: AgeRange::new(|filter, name| {
                map(&filter, name, self.get(&filter).expect("exact age range"))
            }),
            under: UnderAge::new(|filter, name| {
                map(
                    &filter,
                    name,
                    self.get(&filter).expect("under-age threshold"),
                )
            }),
            over: OverAge::new(|filter, name| {
                map(
                    &filter,
                    name,
                    self.get(&filter).expect("over-age threshold"),
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
