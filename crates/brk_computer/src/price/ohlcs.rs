use brk_traversable::Traversable;
use brk_types::{
    Cents, Day1, Day3, Epoch, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30, Month1,
    Month3, Month6, OHLCCents, Version, Week1, Year1, Year10,
};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    BytesVecValue, CachedBoxedVec, Formattable, LazyVec, ReadableCloneableVec, UnaryTransform,
};

use crate::{
    indexes,
    internal::{ComputedVecValue, LazyIndexes, PerResolution},
};

use super::lazy_ohlc::LazyOhlcVec;

#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(merge)]
pub struct LazyOhlcCentsVecs(
    pub  PerResolution<
        LazyOhlcVec<Minute10>,
        LazyOhlcVec<Minute30>,
        LazyOhlcVec<Hour1>,
        LazyOhlcVec<Hour4>,
        LazyOhlcVec<Hour12>,
        LazyOhlcVec<Day1>,
        LazyOhlcVec<Day3>,
        LazyOhlcVec<Week1>,
        LazyOhlcVec<Month1>,
        LazyOhlcVec<Month3>,
        LazyOhlcVec<Month6>,
        LazyOhlcVec<Year1>,
        LazyOhlcVec<Year10>,
        LazyOhlcVec<Halving>,
        LazyOhlcVec<Epoch>,
    >,
);

const COMPUTE_VERSION: Version = Version::TWO;

impl LazyOhlcCentsVecs {
    pub(crate) fn new(
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
        prices: CachedBoxedVec<Height, Cents>,
    ) -> Self {
        let v = version + COMPUTE_VERSION;

        macro_rules! per_period {
            ($field:ident) => {
                LazyOhlcVec::new(name, v, prices.clone(), indexes.$field.first_height.clone())
            };
        }

        Self(PerResolution {
            minute10: per_period!(minute10),
            minute30: per_period!(minute30),
            hour1: per_period!(hour1),
            hour4: per_period!(hour4),
            hour12: per_period!(hour12),
            day1: per_period!(day1),
            day3: per_period!(day3),
            week1: per_period!(week1),
            month1: per_period!(month1),
            month3: per_period!(month3),
            month6: per_period!(month6),
            year1: per_period!(year1),
            year10: per_period!(year10),
            halving: per_period!(halving),
            epoch: per_period!(epoch),
        })
    }
}

impl<T> LazyIndexes<T, OHLCCents>
where
    T: ComputedVecValue + PartialOrd + JsonSchema,
{
    pub(crate) fn from_ohlc_indexes<Transform: UnaryTransform<OHLCCents, T>>(
        name: &str,
        version: Version,
        source: &LazyOhlcCentsVecs,
    ) -> Self {
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

#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(merge)]
pub struct LazyOhlcVecs<T, S>(
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
    T: BytesVecValue + Formattable + Serialize + JsonSchema,
    S: BytesVecValue;

impl<T> LazyOhlcVecs<T, OHLCCents>
where
    T: BytesVecValue + Formattable + Serialize + JsonSchema,
{
    pub(crate) fn from_ohlc_indexes<Transform: UnaryTransform<OHLCCents, T>>(
        name: &str,
        version: Version,
        source: &LazyOhlcCentsVecs,
    ) -> Self {
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
