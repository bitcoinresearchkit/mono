use brk_types::{
    Date, Day1, Day3, Epoch, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30, Month1,
    Month3, Month6, Timestamp, Version, Week1, Year1, Year10,
};
use vecdb::{LazyVec, ReadableBoxedVec, VecIndex};

#[derive(Clone)]
pub struct DailyMappings {
    pub height: LazyVec<Height, Day1, Height, Day1>,
    pub minute10: LazyVec<Minute10, Day1, Minute10, Timestamp>,
    pub minute30: LazyVec<Minute30, Day1, Minute30, Timestamp>,
    pub hour1: LazyVec<Hour1, Day1, Hour1, Timestamp>,
    pub hour4: LazyVec<Hour4, Day1, Hour4, Timestamp>,
    pub hour12: LazyVec<Hour12, Day1, Hour12, Timestamp>,
    pub day3: LazyVec<Day3, Day1, Day3, Date>,
    pub week1: LazyVec<Week1, Day1, Week1, Date>,
    pub month1: LazyVec<Month1, Day1, Month1, Date>,
    pub month3: LazyVec<Month3, Day1, Month3, Date>,
    pub month6: LazyVec<Month6, Day1, Month6, Date>,
    pub year1: LazyVec<Year1, Day1, Year1, Date>,
    pub year10: LazyVec<Year10, Day1, Year10, Date>,
    pub halving: LazyVec<Halving, Day1, Halving, Timestamp>,
    pub epoch: LazyVec<Epoch, Day1, Epoch, Timestamp>,
}

impl DailyMappings {
    pub fn new(indexes: &crate::IndexSources) -> Self {
        let height = LazyVec::init(
            "day1",
            Version::ZERO,
            indexes.height_day1.clone(),
            |_, day| day,
        );

        Self {
            height,
            minute10: timestamp_mapping(indexes.timestamp.minute10.clone()),
            minute30: timestamp_mapping(indexes.timestamp.minute30.clone()),
            hour1: timestamp_mapping(indexes.timestamp.hour1.clone()),
            hour4: timestamp_mapping(indexes.timestamp.hour4.clone()),
            hour12: timestamp_mapping(indexes.timestamp.hour12.clone()),
            day3: date_mapping(indexes.day3_date.clone()),
            week1: date_mapping(indexes.week1_date.clone()),
            month1: date_mapping(indexes.month1_date.clone()),
            month3: date_mapping(indexes.month3_date.clone()),
            month6: date_mapping(indexes.month6_date.clone()),
            year1: date_mapping(indexes.year1_date.clone()),
            year10: date_mapping(indexes.year10_date.clone()),
            halving: timestamp_mapping(indexes.timestamp.halving.clone()),
            epoch: timestamp_mapping(indexes.timestamp.epoch.clone()),
        }
    }
}

fn timestamp_mapping<I: VecIndex>(
    source: ReadableBoxedVec<I, Timestamp>,
) -> LazyVec<I, Day1, I, Timestamp> {
    LazyVec::init("day1", Version::ZERO, source, |_, timestamp| {
        Day1::try_from(Date::from(timestamp)).unwrap_or_default()
    })
}

fn date_mapping<I: VecIndex>(source: ReadableBoxedVec<I, Date>) -> LazyVec<I, Day1, I, Date> {
    LazyVec::init("day1", Version::ZERO, source, |_, date| {
        Day1::try_from(date).unwrap_or_default()
    })
}
