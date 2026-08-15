use brk_traversable::Traversable;
use brk_types::{
    Day1, Day3, Epoch, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30, Month1, Month3,
    Month6, Version, Week1, Year1, Year10,
};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{LazyAggVec, ReadOnlyClone, ReadableBoxedVec, ReadableCloneableVec, VecValue};

use crate::{indexes, internal::PerResolution};

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
    pub(crate) fn from_height_source(
        name: &str,
        height_source: impl ReadableCloneableVec<Height, T> + 'static,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Self {
        Self::from_boxed_height_source(name, Box::new(height_source), version, indexes)
    }

    pub(crate) fn from_boxed_height_source(
        name: &str,
        height_source: ReadableBoxedVec<Height, T>,
        version: Version,
        indexes: &indexes::Vecs,
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
            minute10: res!(indexes.minute10.first_height),
            minute30: res!(indexes.minute30.first_height),
            hour1: res!(indexes.hour1.first_height),
            hour4: res!(indexes.hour4.first_height),
            hour12: res!(indexes.hour12.first_height),
            day1: res!(indexes.day1.first_height),
            day3: res!(indexes.day3.first_height),
            week1: res!(indexes.week1.first_height),
            month1: res!(indexes.month1.first_height),
            month3: res!(indexes.month3.first_height),
            month6: res!(indexes.month6.first_height),
            year1: res!(indexes.year1.first_height),
            year10: res!(indexes.year10.first_height),
            halving: res!(indexes.halving.first_height),
            epoch: res!(indexes.epoch.first_height),
        })
    }
}
