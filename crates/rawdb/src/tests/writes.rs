use std::panic::{AssertUnwindSafe, catch_unwind};
use std::{fs, iter};

use crate::{Database, Error, PAGE_SIZE, Result};

use super::setup_test_db;

#[test]
fn truncating_relocation_preserves_prefix_and_neighbor_on_reopen() -> Result<()> {
    for prefix in [0, 17, PAGE_SIZE] {
        let (db, dir) = setup_test_db()?;
        let region = db.create_region_if_needed("rewrite")?;
        region.write(&vec![3; 2 * PAGE_SIZE])?;
        let old_start = region.meta().start();
        let neighbor = db.create_region_if_needed("neighbor")?;
        neighbor.write(b"kept")?;
        db.flush()?;

        region.truncate_write(prefix, &vec![7; 3 * PAGE_SIZE - prefix])?;
        assert_ne!(region.meta().start(), old_start);
        assert_eq!(db.flush()?, 1);
        drop(region);
        drop(neighbor);
        drop(db);

        let db = Database::open(dir.path())?;
        let region = db.get_region("rewrite").unwrap();
        let reader = region.create_reader();
        assert_eq!(reader.len(), 3 * PAGE_SIZE);
        assert!(reader.read_all()[..prefix].iter().all(|&byte| byte == 3));
        assert!(reader.read_all()[prefix..].iter().all(|&byte| byte == 7));
        assert_eq!(
            db.get_region("neighbor")
                .unwrap()
                .create_reader()
                .read_all(),
            b"kept"
        );
    }
    Ok(())
}

#[test]
fn ordered_batches_handle_empty_single_and_overlapping_writes() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let region = db.create_region_if_needed("batch")?;
    region.write(b"......")?;
    db.flush()?;

    region.batch_write_ordered(iter::empty::<(usize, String)>(), 2, |_, _| {
        panic!("an empty batch must not call the writer")
    });
    assert_eq!(db.flush()?, 0);

    for (values, expected) in [
        (vec![(4, "xy")], b"....xy"),
        (vec![(0, "ab"), (0, "cd"), (1, "ef")], b"cef.xy"),
    ] {
        let mut written = 0;
        let count = values.len();
        region.batch_write_ordered(
            values
                .into_iter()
                .map(|(offset, value)| (offset, value.to_owned())),
            2,
            |value, bytes| {
                bytes.copy_from_slice(value.as_bytes());
                written += 1;
            },
        );
        assert_eq!(written, count);
        assert_eq!(db.flush()?, 1);
        assert_eq!(db.flush()?, 0);
        assert_eq!(region.create_reader().read_all(), expected);
    }
    Ok(())
}

#[test]
fn test_reserve_region_capacity_preserves_data() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("first")?;
    region.write(b"preserved")?;
    let blocker = db.create_region_if_needed("blocker")?;
    let initial_start = region.meta().start();

    region.reserve_capacity(PAGE_SIZE * 3 + 1)?;

    let meta = region.meta();
    assert_ne!(meta.start(), initial_start);
    assert_eq!(meta.byte_len(), b"preserved".len());
    assert_eq!(meta.reserved(), PAGE_SIZE * 4);
    drop(meta);
    assert_eq!(region.create_reader().read_all(), b"preserved");
    assert_eq!(blocker.meta().start(), PAGE_SIZE);

    db.flush()?;
    drop(blocker);
    drop(region);
    drop(db);

    let reopened = Database::open(_temp.path())?;
    let region = reopened.get_region("first").unwrap();
    assert_eq!(region.meta().reserved(), PAGE_SIZE * 4);
    assert_eq!(region.create_reader().read_all(), b"preserved");

    Ok(())
}

#[test]
fn test_truncate_errors() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    region.write(b"Hello")?;

    // Truncating beyond length should error
    let result = region.truncate(10);
    assert!(result.is_err());

    // Truncating to same length should be OK
    let result = region.truncate(5);
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_write_at_boundary_conditions() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("boundary")?;

    // Write at start
    region.write(b"0123456789")?;

    // Write at exact length boundary
    region.write_at(b"ABC", 10)?;

    // Write at position 0
    region.write_at(b"X", 0)?;

    let reader = region.create_reader();
    assert_eq!(reader.read_all(), b"X123456789ABC");

    Ok(())
}

#[test]
fn test_partial_overwrites_data_integrity() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("partial")?;

    // Write initial pattern
    let initial = b"AAAAAAAAAA";
    region.write(initial)?;

    // Overwrite middle
    region.write_at(b"BBB", 3)?;

    // Overwrite start
    region.write_at(b"CC", 0)?;

    // Overwrite end
    region.write_at(b"DD", 8)?;

    let reader = region.create_reader();
    assert_eq!(reader.read_all(), b"CCABBBAADD");

    Ok(())
}

#[test]
fn empty_writes_keep_bounds_errors_and_truncation_semantics() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let region = db.create_region_if_needed("empty_writes")?;
    region.write(b"original")?;
    db.flush()?;
    for truncate in [false, true] {
        let result = if truncate {
            region.truncate_write(9, &[])
        } else {
            region.write_at(&[], 9)
        };
        assert!(matches!(
            result,
            Err(Error::WriteOutOfBounds {
                position: 9,
                region_len: 8
            })
        ));
    }
    region.truncate_write(3, &[])?;
    assert_eq!(region.create_reader().read_all(), b"ori");
    assert_eq!(db.flush()?, 0);
    drop(region);
    drop(db);
    let db = Database::open(_temp.path())?;
    assert_eq!(
        db.get_region("empty_writes")
            .unwrap()
            .create_reader()
            .read_all(),
        b"ori"
    );
    Ok(())
}

#[test]
fn batch_endpoints_match_checked_ranges() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let region = db.create_region_if_needed("bounds")?;
    region.write(&[0; 16])?;
    let offsets = [0, 1, 8, 15, 16, 17, usize::MAX - 1, usize::MAX];
    for width in [0, 1, 8, 16, usize::MAX] {
        for first in offsets {
            for second in offsets {
                let mut expected = [0; 16];
                let valid = first <= second
                    && [first, second].into_iter().all(|offset| {
                        offset
                            .checked_add(width)
                            .is_some_and(|end| end <= expected.len())
                    });
                let accepted = if valid { 2 } else { 0 };
                if valid {
                    expected[first..first + width].fill(1);
                    expected[second..second + width].fill(2);
                }
                region.write_at(&[0; 16], 0)?;
                let mut calls = 0;
                let outcome = catch_unwind(AssertUnwindSafe(|| {
                    region.batch_write_ordered(
                        [(first, ()), (second, ())].into_iter(),
                        width,
                        |_, bytes| {
                            calls += 1;
                            bytes.fill(calls);
                        },
                    );
                }));
                assert_eq!(
                    outcome.is_ok(),
                    accepted == 2,
                    "width={width}, offsets={first},{second}"
                );
                assert_eq!(calls, accepted);
                assert_eq!(region.create_reader().read_all(), expected);
            }
        }
    }
    Ok(())
}

#[test]
fn batch_dirty_span_covers_every_middle_write() -> Result<()> {
    let (db, temp) = setup_test_db()?;
    let region = db.create_region_if_needed("batch")?;
    region.write(&[0; 16])?;
    db.flush()?;
    // Bounds safety and dirty tracking do not depend on interior ordering.
    region.batch_write_ordered(
        [(0, 1u8), (8, 2), (4, 3), (12, 4)].into_iter(),
        1,
        |value, bytes| bytes[0] = *value,
    );
    assert_eq!(db.flush()?, 1);
    drop(region);
    drop(db);
    let db = Database::open(temp.path())?;
    let mut expected = [0; 16];
    for (offset, value) in [(0, 1), (8, 2), (4, 3), (12, 4)] {
        expected[offset] = value;
    }
    assert_eq!(
        db.get_region("batch").unwrap().create_reader().read_all(),
        expected
    );
    Ok(())
}

#[test]
fn test_truncate_write() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    region.write(b"Hello, World!")?;

    let meta_before = region.meta();
    assert_eq!(meta_before.byte_len(), 13);
    drop(meta_before);

    // Truncate write - should set length to exactly the written data
    region.truncate_write(7, b"Rust")?;

    let meta_after = region.meta();
    assert_eq!(meta_after.byte_len(), 11); // 7 + 4
    let start = meta_after.start();
    drop(meta_after);

    let mmap = fs::read(db.path().join("data"))?;
    assert_eq!(&mmap[start..(start + 11)], b"Hello, Rust");

    Ok(())
}

#[test]
fn test_write_at_invalid_position() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    region.write(b"Hello")?;

    // Writing beyond length should fail
    let result = region.write_at(b"World", 10);
    assert!(result.is_err());

    Ok(())
}
