use bitview_traversable::Traversable;
use brk_types::{
    Day1, Day3, Epoch, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30, Month1, Month3,
    Month6, Version, Week1, Year1, Year10,
};
use vecdb::{ReadableBoxedVec, ReadableCloneableVec, VecIndex, VecValue};

use super::{DailyMappings, DailyValue, DailyView, LastDay, RepeatDay};

#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct DailyViews<T>
where
    T: DailyValue,
{
    pub height: DailyView<Height, T, RepeatDay>,
    pub minute10: DailyView<Minute10, T, RepeatDay>,
    pub minute30: DailyView<Minute30, T, RepeatDay>,
    pub hour1: DailyView<Hour1, T, RepeatDay>,
    pub hour4: DailyView<Hour4, T, RepeatDay>,
    pub hour12: DailyView<Hour12, T, RepeatDay>,
    pub day3: DailyView<Day3, T, LastDay>,
    pub week1: DailyView<Week1, T, LastDay>,
    pub month1: DailyView<Month1, T, LastDay>,
    pub month3: DailyView<Month3, T, LastDay>,
    pub month6: DailyView<Month6, T, LastDay>,
    pub year1: DailyView<Year1, T, LastDay>,
    pub year10: DailyView<Year10, T, LastDay>,
    pub halving: DailyView<Halving, T, LastDay>,
    pub epoch: DailyView<Epoch, T, LastDay>,
}

impl<T> DailyViews<T>
where
    T: DailyValue,
{
    pub fn new(
        name: &str,
        source: ReadableBoxedVec<Day1, T>,
        version: Version,
        mappings: &DailyMappings,
    ) -> Self {
        Self {
            height: repeated(name, source.clone(), version, &mappings.height),
            minute10: repeated(name, source.clone(), version, &mappings.minute10),
            minute30: repeated(name, source.clone(), version, &mappings.minute30),
            hour1: repeated(name, source.clone(), version, &mappings.hour1),
            hour4: repeated(name, source.clone(), version, &mappings.hour4),
            hour12: repeated(name, source.clone(), version, &mappings.hour12),
            day3: last(name, source.clone(), version, &mappings.day3),
            week1: last(name, source.clone(), version, &mappings.week1),
            month1: last(name, source.clone(), version, &mappings.month1),
            month3: last(name, source.clone(), version, &mappings.month3),
            month6: last(name, source.clone(), version, &mappings.month6),
            year1: last(name, source.clone(), version, &mappings.year1),
            year10: last(name, source.clone(), version, &mappings.year10),
            halving: last(name, source.clone(), version, &mappings.halving),
            epoch: last(name, source, version, &mappings.epoch),
        }
    }
}

fn repeated<I, T, V>(
    name: &str,
    source: ReadableBoxedVec<Day1, T>,
    version: Version,
    mapping: &V,
) -> DailyView<I, T, RepeatDay>
where
    I: VecIndex,
    T: VecValue,
    V: ReadableCloneableVec<I, Day1> + ?Sized,
{
    DailyView::new(name, version, source, mapping.read_only_boxed_clone())
}

fn last<I, T, V>(
    name: &str,
    source: ReadableBoxedVec<Day1, T>,
    version: Version,
    mapping: &V,
) -> DailyView<I, T, LastDay>
where
    I: VecIndex,
    T: VecValue,
    V: ReadableCloneableVec<I, Day1> + ?Sized,
{
    DailyView::new(name, version, source, mapping.read_only_boxed_clone())
}
