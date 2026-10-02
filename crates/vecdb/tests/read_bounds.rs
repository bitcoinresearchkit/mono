#![cfg(feature = "serde")]

use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    thread,
};

use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, Database, ImportableVec, ReadBounds, ValueWriter, Version,
    WritableVec,
};

#[test]
fn explicit_bounds_cover_lengths_ranges_json_csv_and_writer_lifetimes() {
    let temp = tempdir().unwrap();
    let db = Database::open(temp.path()).unwrap();
    let mut source: BytesVec<usize, u64> =
        BytesVec::forced_import(&db, "source", Version::ONE).unwrap();
    for value in [10, 20, 30, 40] {
        source.push(value);
    }
    source.flush().unwrap();

    let mut bounds = ReadBounds::new();
    assert!(
        bounds.bind(&source).is_none(),
        "missing bounds must not expose the full vector"
    );
    bounds.set("usize", 2);
    let bounded = bounds.bind(&source).unwrap();
    assert_eq!(bounded.visible_len(), 2);
    assert_eq!(bounded.range_count(None, None), 2);
    assert_eq!(bounded.range_count(Some(-1), None), 1);
    assert_eq!(bounded.range_count(Some(99), Some(1)), 0);
    assert_eq!(bounded.range_weight(None, None), 2 * size_of::<u64>());
    let mut json = Vec::new();
    bounded
        .write_json(None, Some(usize::MAX), &mut json)
        .unwrap();
    assert_eq!(json, b"[10,20]");
    json.clear();
    bounded.write_json_value_at(2, &mut json).unwrap();
    assert!(json.is_empty());
    bounded.write_json_value_at(1, &mut json).unwrap();
    assert_eq!(json, b"20");
    let mut csv = String::new();
    bounded
        .write_csv_column(None, Some(usize::MAX), &mut csv)
        .unwrap();
    assert_eq!(csv, "10\n20\n");
    let mut writer = bounded.create_writer(Some(-1), None);
    csv.clear();
    writer.write_next(&mut csv).unwrap();
    assert_eq!(csv, "20");
    assert!(writer.write_next(&mut csv).is_err());
    assert_eq!(
        source.visible_len(),
        4,
        "bounded operations must restore their caller's scope"
    );
}

#[test]
fn bounds_restore_after_unwinding_and_do_not_leak_to_other_threads() {
    let temp = tempdir().unwrap();
    let db = Database::open(temp.path()).unwrap();
    let mut source: BytesVec<usize, u64> =
        BytesVec::forced_import(&db, "source", Version::ONE).unwrap();
    for value in [10, 20, 30] {
        source.push(value);
    }
    source.flush().unwrap();
    let mut bounds = ReadBounds::new();
    bounds.set("usize", 1);
    assert!(
        catch_unwind(AssertUnwindSafe(|| bounds.scope(|| {
            assert_eq!(source.visible_len(), 1);
            thread::scope(|threads| {
                assert_eq!(threads.spawn(|| source.visible_len()).join().unwrap(), 3);
            });
            panic!("unwind the read scope");
        })))
        .is_err()
    );
    assert_eq!(source.visible_len(), 3);
}
