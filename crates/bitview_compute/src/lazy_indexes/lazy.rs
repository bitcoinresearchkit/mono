use bitview_traversable::Traversable;
use brk_types::{
    Day1, Day3, Epoch, Halving, Hour1, Hour4, Hour12, Minute10, Minute30, Month1, Month3, Month6,
    Version, Week1, Year1, Year10,
};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{LazyVec, ReadableCloneableVec, UnaryTransform, VecValue};

use crate::{ComputedVecValue, PerResolution};

#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyIndexes<T, S>(
    pub  PerResolution<
        LazyVec<Minute10, T, Minute10, S>,
        LazyVec<Minute30, T, Minute30, S>,
        LazyVec<Hour1, T, Hour1, S>,
        LazyVec<Hour4, T, Hour4, S>,
        LazyVec<Hour12, T, Hour12, S>,
        LazyVec<Day1, T, Day1, S>,
        LazyVec<Day3, T, Day3, S>,
        LazyVec<Week1, T, Week1, S>,
        LazyVec<Month1, T, Month1, S>,
        LazyVec<Month3, T, Month3, S>,
        LazyVec<Month6, T, Month6, S>,
        LazyVec<Year1, T, Year1, S>,
        LazyVec<Year10, T, Year10, S>,
        LazyVec<Halving, T, Halving, S>,
        LazyVec<Epoch, T, Epoch, S>,
    >,
)
where
    T: ComputedVecValue + PartialOrd + JsonSchema,
    S: VecValue;

impl<T, S> LazyIndexes<T, S>
where
    T: ComputedVecValue + PartialOrd + JsonSchema,
    S: VecValue,
{
    pub fn from_lazy_indexes<Transform, U>(
        name: &str,
        version: Version,
        source: &LazyIndexes<S, U>,
    ) -> Self
    where
        Transform: UnaryTransform<S, T>,
        S: ComputedVecValue + PartialOrd + JsonSchema,
        U: VecValue,
    {
        macro_rules! period {
            ($idx:ident) => {
                LazyVec::transformed::<Transform>(
                    name,
                    version,
                    source.$idx.read_only_boxed_clone(),
                )
            };
        }

        Self(PerResolution {
            minute10: period!(minute10),
            minute30: period!(minute30),
            hour1: period!(hour1),
            hour4: period!(hour4),
            hour12: period!(hour12),
            day1: period!(day1),
            day3: period!(day3),
            week1: period!(week1),
            month1: period!(month1),
            month3: period!(month3),
            month6: period!(month6),
            year1: period!(year1),
            year10: period!(year10),
            halving: period!(halving),
            epoch: period!(epoch),
        })
    }
}
