use brk_types::{Day1, Height, StoredBool, StoredF64, Version};
use vecdb::{
    AnyStoredVec, Database, EagerVec, ImportableVec, PcoVec, ReadableCloneableVec, ReadableVec,
    WritableVec,
};

use super::{DailyView, RepeatDay, last::last_source_index, repeat::repeated_source_index};

#[test]
fn repeat_uses_the_same_daily_value_throughout_the_day() {
    let mapping = [Day1::from(0), Day1::from(0), Day1::from(1)];

    assert_eq!(repeated_source_index(&mapping, 0, 2), Some(0));
    assert_eq!(repeated_source_index(&mapping, 1, 2), Some(0));
    assert_eq!(repeated_source_index(&mapping, 2, 2), Some(1));
    assert_eq!(repeated_source_index(&mapping, 2, 1), None);
}

#[test]
fn coarser_period_uses_its_last_available_day() {
    let mapping = [Day1::from(0), Day1::from(3), Day1::from(6)];

    assert_eq!(last_source_index(&mapping, 0, 8), Some(2));
    assert_eq!(last_source_index(&mapping, 1, 8), Some(5));
    assert_eq!(last_source_index(&mapping, 2, 8), Some(7));
    assert_eq!(last_source_index(&mapping, 1, 5), Some(4));
    assert_eq!(last_source_index(&mapping, 2, 5), None);
}

#[test]
fn repeated_view_maps_ranges_and_preserves_missing_days() {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("brk-daily-view-{}-{suffix}", std::process::id()));
    let db = Database::open(&path).unwrap();

    let mut source: EagerVec<PcoVec<Day1, StoredF64>> =
        EagerVec::forced_import(&db, "source", Version::ONE).unwrap();
    let mut mapping: EagerVec<PcoVec<Height, Day1>> =
        EagerVec::forced_import(&db, "mapping", Version::ONE).unwrap();
    for value in [10.0, 20.0, 30.0] {
        source.push(StoredF64::from(value));
    }
    for day in [0, 0, 1, 2, 3] {
        mapping.push(Day1::from(day));
    }
    source.write().unwrap();
    mapping.write().unwrap();

    let view = DailyView::<Height, StoredF64, RepeatDay>::new(
        "test",
        Version::ONE,
        source.read_only_boxed_clone(),
        mapping.read_only_boxed_clone(),
    );

    assert_eq!(
        view.collect_range_at(0, 5),
        vec![
            Some(StoredF64::from(10.0)),
            Some(StoredF64::from(10.0)),
            Some(StoredF64::from(20.0)),
            Some(StoredF64::from(30.0)),
            None,
        ]
    );

    drop(view);
    drop(mapping);
    drop(source);
    drop(db);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn repeated_view_supports_stored_booleans() {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("brk-daily-bool-{}-{suffix}", std::process::id()));
    let db = Database::open(&path).unwrap();

    let mut source: EagerVec<PcoVec<Day1, StoredBool>> =
        EagerVec::forced_import(&db, "source", Version::ONE).unwrap();
    let mut mapping: EagerVec<PcoVec<Height, Day1>> =
        EagerVec::forced_import(&db, "mapping", Version::ONE).unwrap();
    source.push(StoredBool::FALSE);
    source.push(StoredBool::TRUE);
    for day in [0, 0, 1, 1] {
        mapping.push(Day1::from(day));
    }
    source.write().unwrap();
    mapping.write().unwrap();

    let view = DailyView::<Height, StoredBool, RepeatDay>::new(
        "test",
        Version::ONE,
        source.read_only_boxed_clone(),
        mapping.read_only_boxed_clone(),
    );

    assert_eq!(
        view.collect_range_at(0, 4),
        vec![
            Some(StoredBool::FALSE),
            Some(StoredBool::FALSE),
            Some(StoredBool::TRUE),
            Some(StoredBool::TRUE),
        ]
    );

    drop(view);
    drop(mapping);
    drop(source);
    drop(db);
    std::fs::remove_dir_all(path).unwrap();
}
