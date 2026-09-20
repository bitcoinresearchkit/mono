use bitview_collections::PerResolution;
use bitview_vecs::{IndexSources, LazyPreviousDeltaVec, RangeMapVec};
use brk_types::{Height, Version};
use rangeindex::SharedRangeMap;
use vecdb::{
    AnyStoredVec, Budgeted, Database, EagerVec, ImportableVec, PcoVec, PcoVecValue,
    ReadableCloneableVec, VecIndex, WritableVec,
};

mod cache;
pub(crate) use cache::init_cache;

pub fn stored<I: VecIndex, T: PcoVecValue>(
    db: &Database,
    name: &str,
    values: impl IntoIterator<Item = T>,
) -> EagerVec<PcoVec<I, T, Budgeted>> {
    init_cache();
    let mut vec = EagerVec::forced_import(db, name, Version::ONE).unwrap();
    for value in values {
        vec.push(value);
    }
    vec.write().unwrap();
    vec
}

pub fn first_heights<I: VecIndex>(
    name: &str,
    values: impl IntoIterator<Item = Height>,
) -> RangeMapVec<I, Height> {
    RangeMapVec::new(
        name,
        Version::ONE,
        SharedRangeMap::new(values.into_iter().collect()),
    )
}

pub fn indexes(db: &Database) -> IndexSources {
    macro_rules! empty {
        ($name:expr) => {
            stored(db, $name, std::iter::empty())
        };
    }
    macro_rules! resolutions {
        ($prefix:literal, $make:expr) => {
            PerResolution {
                minute10: $make(concat!($prefix, "_minute10")),
                minute30: $make(concat!($prefix, "_minute30")),
                hour1: $make(concat!($prefix, "_hour1")),
                hour4: $make(concat!($prefix, "_hour4")),
                hour12: $make(concat!($prefix, "_hour12")),
                day1: $make(concat!($prefix, "_day1")),
                day3: $make(concat!($prefix, "_day3")),
                week1: $make(concat!($prefix, "_week1")),
                month1: $make(concat!($prefix, "_month1")),
                month3: $make(concat!($prefix, "_month3")),
                month6: $make(concat!($prefix, "_month6")),
                year1: $make(concat!($prefix, "_year1")),
                year10: $make(concat!($prefix, "_year10")),
                halving: $make(concat!($prefix, "_halving")),
                epoch: $make(concat!($prefix, "_epoch")),
            }
        };
    }
    IndexSources {
        first_height: resolutions!("first", |name| first_heights(name, [])),
        timestamp: resolutions!("timestamp", |name| empty!(name).read_only_boxed_clone()),
        height_minute10: empty!("height_minute10").read_only_boxed_clone(),
        height_day1: empty!("height_day1").read_only_boxed_clone(),
        height_tx_index_count: LazyPreviousDeltaVec::new(
            "tx_count",
            Version::ONE,
            &empty!("tx_cumulative"),
        ),
        day3_date: empty!("day3_date").read_only_boxed_clone(),
        week1_date: empty!("week1_date").read_only_boxed_clone(),
        month1_date: empty!("month1_date").read_only_boxed_clone(),
        month3_date: empty!("month3_date").read_only_boxed_clone(),
        month6_date: empty!("month6_date").read_only_boxed_clone(),
        year1_date: empty!("year1_date").read_only_boxed_clone(),
        year10_date: empty!("year10_date").read_only_boxed_clone(),
    }
}
