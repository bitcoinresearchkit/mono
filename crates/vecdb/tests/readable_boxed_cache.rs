#![cfg(feature = "pco")]
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, Budgeted, Database, EagerVec, ImportableVec, PcoVec, ReadOnlyClone,
    ReadableCloneableVec, ReadableVec, Stamp, Version, WritableVec,
};

#[test]
fn captured_readers_share_appends_and_source_owned_invalidation() {
    init_cache();
    let dir = tempdir().unwrap();
    let db = Database::open(dir.path()).unwrap();
    let mut values =
        EagerVec::<PcoVec<usize, u64, Budgeted>>::import(&db, "prices", Version::ONE).unwrap();
    values.push(10);
    values.push(20);
    values.write().unwrap();
    let reader = values.read_only_clone();
    let captured = values.read_only_boxed_clone();
    let previous = reader.collect();
    assert_eq!(previous, [10, 20]);
    assert!(captured.read_cached_into_at(0, 2, &mut Vec::new()));
    for length in [2, 3] {
        values.truncate_if_needed_at(length).unwrap();
        assert!(reader.read_cached_into_at(0, 2, &mut Vec::new()));
    }
    let stamp = Stamp::from(42_u64);
    values.truncate_if_needed_with_stamp(2, stamp).unwrap();
    assert_eq!(values.stamp(), stamp);
    values.push(40);
    values.write().unwrap();
    assert!(reader.read_cached_into_at(0, 2, &mut Vec::new()));
    assert!(reader.read_cached_into_at(2, 3, &mut Vec::new()));
    assert_eq!(captured.collect(), [10, 20, 40]);
    values.truncate_if_needed_at(1).unwrap();
    assert!(captured.read_cached_into_at(0, 1, &mut Vec::new()));
    assert!(!captured.read_cached_into_at(1, 2, &mut Vec::new()));
    values.push(30);
    values.push(50);
    values.write().unwrap();
    assert_eq!(reader.collect(), [10, 30, 50]);
    assert_eq!(captured.collect(), [10, 30, 50]);
    assert_eq!(previous, [10, 20], "caller-owned results do not change");
    assert!(values.read_cached_into_at(0, 3, &mut Vec::new()));
}

#[allow(dead_code)]
#[path = "common/cache.rs"]
mod cache;
use cache::init_cache;
