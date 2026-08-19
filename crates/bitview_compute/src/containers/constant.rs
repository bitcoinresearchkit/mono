use bitview_traversable::Traversable;
use brk_types::{
    Day1, Day3, Epoch, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30, Month1, Month3,
    Month6, Version, Week1, Year1, Year10,
};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{Formattable, LazyVec, UnaryTransform, VecValue};

#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct ConstantVecs<T>
where
    T: VecValue + Formattable + Serialize + JsonSchema,
{
    pub height: LazyVec<Height, T, Height, Minute10>,
    pub minute10: LazyVec<Minute10, T, Minute10, Height>,
    pub minute30: LazyVec<Minute30, T, Minute30, Height>,
    pub hour1: LazyVec<Hour1, T, Hour1, Height>,
    pub hour4: LazyVec<Hour4, T, Hour4, Height>,
    pub hour12: LazyVec<Hour12, T, Hour12, Height>,
    pub day1: LazyVec<Day1, T, Day1, Height>,
    pub day3: LazyVec<Day3, T, Day3, Height>,
    pub week1: LazyVec<Week1, T, Week1, Height>,
    pub month1: LazyVec<Month1, T, Month1, Height>,
    pub month3: LazyVec<Month3, T, Month3, Height>,
    pub month6: LazyVec<Month6, T, Month6, Height>,
    pub year1: LazyVec<Year1, T, Year1, Height>,
    pub year10: LazyVec<Year10, T, Year10, Height>,
    pub halving: LazyVec<Halving, T, Halving, Height>,
    pub epoch: LazyVec<Epoch, T, Epoch, Height>,
}

impl<T: VecValue + Formattable + Serialize + JsonSchema> ConstantVecs<T> {
    pub fn new<F>(name: &str, version: Version, indexes: &crate::IndexSources) -> Self
    where
        F: UnaryTransform<Height, T>
            + UnaryTransform<Minute10, T>
            + UnaryTransform<Minute30, T>
            + UnaryTransform<Hour1, T>
            + UnaryTransform<Hour4, T>
            + UnaryTransform<Hour12, T>
            + UnaryTransform<Day1, T>
            + UnaryTransform<Day3, T>
            + UnaryTransform<Week1, T>
            + UnaryTransform<Month1, T>
            + UnaryTransform<Month3, T>
            + UnaryTransform<Month6, T>
            + UnaryTransform<Year1, T>
            + UnaryTransform<Year10, T>
            + UnaryTransform<Halving, T>
            + UnaryTransform<Epoch, T>,
    {
        macro_rules! period {
            ($idx:ident) => {
                LazyVec::init(
                    name,
                    version,
                    indexes.first_height.$idx.clone(),
                    |idx, _: Height| F::apply(idx),
                )
            };
        }

        Self {
            height: LazyVec::init(name, version, indexes.height_minute10.clone(), |idx, _| {
                F::apply(idx)
            }),
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
        }
    }
}
