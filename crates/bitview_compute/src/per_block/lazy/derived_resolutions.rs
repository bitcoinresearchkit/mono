use bitview_traversable::Traversable;
use brk_types::{
    Day1, Day3, Epoch, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30, Month1, Month3,
    Month6, Version, Week1, Year1, Year10,
};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{ReadableBoxedVec, ReadableCloneableVec, UnaryTransform, VecValue};

use crate::{ComputedVecValue, NumericValue, PerResolution, Resolutions};

use super::{LazyTransformLast, MapOption};

#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct DerivedResolutions<T, S1T = T>(
    pub  PerResolution<
        LazyTransformLast<Minute10, Option<T>, Option<S1T>>,
        LazyTransformLast<Minute30, Option<T>, Option<S1T>>,
        LazyTransformLast<Hour1, Option<T>, Option<S1T>>,
        LazyTransformLast<Hour4, Option<T>, Option<S1T>>,
        LazyTransformLast<Hour12, Option<T>, Option<S1T>>,
        LazyTransformLast<Day1, Option<T>, Option<S1T>>,
        LazyTransformLast<Day3, Option<T>, Option<S1T>>,
        LazyTransformLast<Week1, Option<T>, Option<S1T>>,
        LazyTransformLast<Month1, Option<T>, Option<S1T>>,
        LazyTransformLast<Month3, Option<T>, Option<S1T>>,
        LazyTransformLast<Month6, Option<T>, Option<S1T>>,
        LazyTransformLast<Year1, Option<T>, Option<S1T>>,
        LazyTransformLast<Year10, Option<T>, Option<S1T>>,
        LazyTransformLast<Halving, T, S1T>,
        LazyTransformLast<Epoch, T, S1T>,
    >,
)
where
    T: VecValue + PartialOrd + JsonSchema,
    S1T: VecValue;

impl<T, S1T> DerivedResolutions<T, S1T>
where
    T: VecValue + PartialOrd + JsonSchema + 'static,
    S1T: VecValue + PartialOrd + JsonSchema,
{
    pub fn from_height_source<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        height_source: ReadableBoxedVec<Height, S1T>,
        indexes: &crate::IndexSources,
    ) -> Self
    where
        S1T: NumericValue,
    {
        let derived = Resolutions::from_boxed_height_source(name, height_source, version, indexes);
        Self::from_derived_computed::<F>(name, version, &derived)
    }

    pub fn from_derived_computed<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        source: &Resolutions<S1T>,
    ) -> Self {
        macro_rules! period {
            ($p:ident) => {
                LazyTransformLast::from_boxed::<MapOption<F>>(
                    name,
                    version,
                    source.$p.read_only_boxed_clone(),
                )
            };
        }

        macro_rules! epoch {
            ($p:ident) => {
                LazyTransformLast::from_boxed::<F>(name, version, source.$p.read_only_boxed_clone())
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
            halving: epoch!(halving),
            epoch: epoch!(epoch),
        })
    }

    pub fn from_lazy<F, S2T>(
        name: &str,
        version: Version,
        source: &DerivedResolutions<S1T, S2T>,
    ) -> Self
    where
        F: UnaryTransform<S1T, T>,
        S2T: ComputedVecValue + JsonSchema,
    {
        macro_rules! period {
            ($p:ident) => {
                LazyTransformLast::from_boxed::<MapOption<F>>(
                    name,
                    version,
                    source.$p.read_only_boxed_clone(),
                )
            };
        }

        macro_rules! epoch {
            ($p:ident) => {
                LazyTransformLast::from_boxed::<F>(name, version, source.$p.read_only_boxed_clone())
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
            halving: epoch!(halving),
            epoch: epoch!(epoch),
        })
    }
}
