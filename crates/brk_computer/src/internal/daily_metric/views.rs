use brk_traversable::Traversable;
use brk_types::{
    Day1, Day3, Epoch, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30, Month1, Month3,
    Month6, Version, Week1, Year1, Year10,
};
use vecdb::{ReadableBoxedVec, ReadableCloneableVec, VecIndex, VecValue};

use super::{DailyMappings, DailyValue, DailyView, LastDay, RepeatDay};

type Repeated<I, T> = DailyView<I, T, RepeatDay>;
type Last<I, T> = DailyView<I, T, LastDay>;

#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct DailyViews<T>
where
    T: DailyValue,
{
    pub height: Repeated<Height, T>,
    pub minute10: Repeated<Minute10, T>,
    pub minute30: Repeated<Minute30, T>,
    pub hour1: Repeated<Hour1, T>,
    pub hour4: Repeated<Hour4, T>,
    pub hour12: Repeated<Hour12, T>,
    pub day3: Last<Day3, T>,
    pub week1: Last<Week1, T>,
    pub month1: Last<Month1, T>,
    pub month3: Last<Month3, T>,
    pub month6: Last<Month6, T>,
    pub year1: Last<Year1, T>,
    pub year10: Last<Year10, T>,
    pub halving: Last<Halving, T>,
    pub epoch: Last<Epoch, T>,
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
) -> Repeated<I, T>
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
) -> Last<I, T>
where
    I: VecIndex,
    T: VecValue,
    V: ReadableCloneableVec<I, Day1> + ?Sized,
{
    DailyView::new(name, version, source, mapping.read_only_boxed_clone())
}
