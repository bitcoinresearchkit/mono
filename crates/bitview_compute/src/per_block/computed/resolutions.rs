use bitview_traversable::Traversable;
use brk_types::{
    Day1, Day3, Epoch, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30, Month1, Month3,
    Month6, Version, Week1, Year1, Year10,
};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{LazyAggVec, ReadOnlyClone, ReadableBoxedVec, ReadableCloneableVec, VecValue};

use crate::PerResolution;

use super::CoarserIndex;

#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct Resolutions<T>(
    pub  PerResolution<
        LazyAggVec<Minute10, Option<T>, Height, Height, T>,
        LazyAggVec<Minute30, Option<T>, Height, Height, T>,
        LazyAggVec<Hour1, Option<T>, Height, Height, T>,
        LazyAggVec<Hour4, Option<T>, Height, Height, T>,
        LazyAggVec<Hour12, Option<T>, Height, Height, T>,
        LazyAggVec<Day1, Option<T>, Height, Height, T>,
        LazyAggVec<Day3, Option<T>, Height, Height, T>,
        LazyAggVec<Week1, Option<T>, Height, Height, T>,
        LazyAggVec<Month1, Option<T>, Height, Height, T>,
        LazyAggVec<Month3, Option<T>, Height, Height, T>,
        LazyAggVec<Month6, Option<T>, Height, Height, T>,
        LazyAggVec<Year1, Option<T>, Height, Height, T>,
        LazyAggVec<Year10, Option<T>, Height, Height, T>,
        LazyAggVec<Halving, T, Height, Height, T, CoarserIndex<Halving>>,
        LazyAggVec<Epoch, T, Height, Height, T, CoarserIndex<Epoch>>,
    >,
)
where
    T: VecValue + PartialOrd + JsonSchema;

impl<T> ReadOnlyClone for Resolutions<T>
where
    T: VecValue + PartialOrd + JsonSchema,
{
    type ReadOnly = Self;

    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}

impl<T> Resolutions<T>
where
    T: VecValue + PartialOrd + JsonSchema + 'static,
{
    pub fn from_height_source(
        name: &str,
        height_source: impl ReadableCloneableVec<Height, T> + 'static,
        version: Version,
        indexes: &crate::IndexSources,
    ) -> Self {
        Self::from_boxed_height_source(name, ReadableBoxedVec::new(height_source), version, indexes)
    }

    pub fn from_boxed_height_source(
        name: &str,
        height_source: ReadableBoxedVec<Height, T>,
        version: Version,
        indexes: &crate::IndexSources,
    ) -> Self {
        macro_rules! res {
            ($field:expr) => {{
                let cached = $field.clone();
                let mapping_version = cached.version();
                LazyAggVec::new(
                    name,
                    version,
                    mapping_version,
                    height_source.clone(),
                    move || cached.snapshot(),
                )
            }};
        }

        Self(PerResolution {
            minute10: res!(indexes.cached_first_height.minute10),
            minute30: res!(indexes.cached_first_height.minute30),
            hour1: res!(indexes.cached_first_height.hour1),
            hour4: res!(indexes.cached_first_height.hour4),
            hour12: res!(indexes.cached_first_height.hour12),
            day1: res!(indexes.cached_first_height.day1),
            day3: res!(indexes.cached_first_height.day3),
            week1: res!(indexes.cached_first_height.week1),
            month1: res!(indexes.cached_first_height.month1),
            month3: res!(indexes.cached_first_height.month3),
            month6: res!(indexes.cached_first_height.month6),
            year1: res!(indexes.cached_first_height.year1),
            year10: res!(indexes.cached_first_height.year10),
            halving: res!(indexes.cached_first_height.halving),
            epoch: res!(indexes.cached_first_height.epoch),
        })
    }
}
