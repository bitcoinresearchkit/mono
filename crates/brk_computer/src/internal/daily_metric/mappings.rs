use brk_types::{
    Date, Day1, Day3, Epoch, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30, Month1,
    Month3, Month6, Timestamp, Version, Week1, Year1, Year10,
};
use vecdb::{LazyVec, ReadableBoxedVec, ReadableCloneableVec, VecIndex};

use crate::indexes;

type DayMapping<I, T> = LazyVec<I, Day1, I, T>;

#[derive(Clone)]
pub struct DailyMappings {
    pub height: DayMapping<Height, Day1>,
    pub minute10: DayMapping<Minute10, Timestamp>,
    pub minute30: DayMapping<Minute30, Timestamp>,
    pub hour1: DayMapping<Hour1, Timestamp>,
    pub hour4: DayMapping<Hour4, Timestamp>,
    pub hour12: DayMapping<Hour12, Timestamp>,
    pub day3: DayMapping<Day3, Date>,
    pub week1: DayMapping<Week1, Date>,
    pub month1: DayMapping<Month1, Date>,
    pub month3: DayMapping<Month3, Date>,
    pub month6: DayMapping<Month6, Date>,
    pub year1: DayMapping<Year1, Date>,
    pub year10: DayMapping<Year10, Date>,
    pub halving: DayMapping<Halving, Timestamp>,
    pub epoch: DayMapping<Epoch, Timestamp>,
}

impl DailyMappings {
    pub fn new(indexes: &indexes::Vecs) -> Self {
        let height = LazyVec::init(
            "day1",
            Version::ZERO,
            indexes.height.day1_read_only_boxed_clone(),
            |_, day| day,
        );

        Self {
            height,
            minute10: timestamp_mapping(indexes.timestamp.minute10.read_only_boxed_clone()),
            minute30: timestamp_mapping(indexes.timestamp.minute30.read_only_boxed_clone()),
            hour1: timestamp_mapping(indexes.timestamp.hour1.read_only_boxed_clone()),
            hour4: timestamp_mapping(indexes.timestamp.hour4.read_only_boxed_clone()),
            hour12: timestamp_mapping(indexes.timestamp.hour12.read_only_boxed_clone()),
            day3: date_mapping(indexes.day3.date.read_only_boxed_clone()),
            week1: date_mapping(indexes.week1.date.read_only_boxed_clone()),
            month1: date_mapping(indexes.month1.date.read_only_boxed_clone()),
            month3: date_mapping(indexes.month3.date.read_only_boxed_clone()),
            month6: date_mapping(indexes.month6.date.read_only_boxed_clone()),
            year1: date_mapping(indexes.year1.date.read_only_boxed_clone()),
            year10: date_mapping(indexes.year10.date.read_only_boxed_clone()),
            halving: timestamp_mapping(indexes.timestamp.halving.read_only_boxed_clone()),
            epoch: timestamp_mapping(indexes.timestamp.epoch.read_only_boxed_clone()),
        }
    }
}

fn timestamp_mapping<I: VecIndex>(
    source: ReadableBoxedVec<I, Timestamp>,
) -> DayMapping<I, Timestamp> {
    LazyVec::init("day1", Version::ZERO, source, |_, timestamp| {
        Day1::try_from(Date::from(timestamp)).unwrap_or_default()
    })
}

fn date_mapping<I: VecIndex>(source: ReadableBoxedVec<I, Date>) -> DayMapping<I, Date> {
    LazyVec::init("day1", Version::ZERO, source, |_, date| {
        Day1::try_from(date).unwrap_or_default()
    })
}
