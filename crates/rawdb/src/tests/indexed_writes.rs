use std::{
    collections::BTreeMap,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::mpsc,
    thread,
    time::Duration,
};

use crate::{Database, Result};

use super::setup_test_db;

#[test]
fn indexed_batches_match_record_bounds() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let neighbor = db.create_region_if_needed("neighbor")?;
    neighbor.write(b"preserved")?;
    let region = db.create_region_if_needed("records")?;
    region.write(&[0; 16])?;
    assert_ne!(region.meta().start(), 0);
    let indices = [0, 1, 3, 7, 16, usize::MAX / 2, usize::MAX];
    for at in [0, 1, 8, 16, 17, usize::MAX] {
        for width in [0, 1, 2, 8, usize::MAX] {
            for first in indices {
                for last in indices {
                    let values = BTreeMap::from([(first, 1u8), (last, 2)]);
                    // Use wider arithmetic as an independent oracle, including
                    // zero-width records with keys near usize::MAX.
                    let valid = values
                        .keys()
                        .all(|&index| at as u128 + (index as u128 + 1) * width as u128 <= 16);
                    let mut expected = [0; 16];
                    if valid {
                        for (&index, &value) in &values {
                            let offset = at + index * width;
                            expected[offset..offset + width].fill(value);
                        }
                    }
                    let count = values.len();
                    region.write_at(&[0; 16], 0)?;
                    let mut calls = 0;
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        region.write_indexed(values, width, at, |value, bytes| {
                            calls += 1;
                            bytes.fill(*value);
                        });
                    }));
                    assert_eq!(
                        result.is_ok(),
                        valid,
                        "at={at}, width={width}, keys={first},{last}"
                    );
                    assert_eq!(calls, if valid { count } else { 0 });
                    assert_eq!(region.create_reader().read_all(), expected);
                }
            }
        }
    }
    assert_eq!(neighbor.create_reader().read_all(), b"preserved");
    Ok(())
}

#[test]
fn indexed_empty_and_invalid_batches_do_not_dirty_data() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let region = db.create_region_if_needed("records")?;
    region.write(&[0; 16])?;
    db.flush()?;
    region.write_indexed(
        BTreeMap::<usize, u8>::new(),
        usize::MAX,
        usize::MAX,
        |_, _| {
            panic!("empty batch must not call the writer");
        },
    );
    assert_eq!(db.flush()?, 0);
    for (last, width, at) in [
        (2, 8, 0),
        (1, 8, 1),
        (usize::MAX, 2, 0),
        (0, 1, usize::MAX),
        (1, usize::MAX, 0),
        (0, 0, 17),
    ] {
        let mut calls = 0;
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                region.write_indexed(BTreeMap::from([(0, 7u8), (last, 9)]), width, at, |_, _| {
                    calls += 1
                });
            }))
            .is_err()
        );
        assert_eq!(calls, 0);
        assert_eq!(db.flush()?, 0);
    }
    region.write_indexed(
        BTreeMap::from([(0, ()), (usize::MAX, ())]),
        0,
        16,
        |_, bytes| assert!(bytes.is_empty()),
    );
    assert_eq!(db.flush()?, 0);
    assert_eq!(region.create_reader().read_all(), &[0; 16]);
    Ok(())
}

#[test]
fn indexed_batch_preserves_order_and_panic_writes_on_reopen() -> Result<()> {
    let (db, temp) = setup_test_db()?;
    let region = db.create_region_if_needed("records")?;
    region.write(&[0; 16])?;
    db.flush()?;
    let mut visited = Vec::new();
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            region.write_indexed(
                BTreeMap::from([(5, 11u8), (0, 7), (3, 9)]),
                2,
                1,
                |value, bytes| {
                    visited.push(*value);
                    bytes.fill(*value);
                    assert_ne!(*value, 9, "callback failed after writing");
                },
            );
        }))
        .is_err()
    );
    assert_eq!(visited, [7, 9]);
    assert_eq!(db.flush()?, 1);
    drop(region);
    drop(db);
    let db = Database::open(temp.path())?;
    let mut expected = [0; 16];
    expected[1..3].fill(7);
    expected[7..9].fill(9);
    assert_eq!(
        db.get_region("records").unwrap().create_reader().read_all(),
        expected
    );
    Ok(())
}

struct DropFailure(bool);

impl Drop for DropFailure {
    fn drop(&mut self) {
        assert!(!self.0, "value drop failed");
    }
}

#[test]
fn indexed_value_drop_panic_preserves_dirty_tracking() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let region = db.create_region_if_needed("records")?;
    region.write(&[0; 16])?;
    db.flush()?;
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            region.write_indexed(
                BTreeMap::from([(0, DropFailure(true)), (1, DropFailure(false))]),
                8,
                0,
                |_, bytes| bytes.fill(7),
            );
        }))
        .is_err()
    );
    assert_eq!(db.flush()?, 1);
    assert_eq!(
        region.create_reader().read_all(),
        &[7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(db.flush()?, 0);
    Ok(())
}

#[test]
fn indexed_callback_excludes_readers_and_flush() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let region = db.create_region_if_needed("records")?;
    region.write(&[0; 8])?;
    db.flush()?;
    let output = region.clone();
    let (entered, ready) = mpsc::channel();
    let (release, proceed) = mpsc::channel();
    let writer = thread::spawn(move || {
        output.write_indexed(BTreeMap::from([(0, 7u8)]), 8, 0, |value, bytes| {
            bytes.fill(*value);
            entered.send(()).unwrap();
            proceed.recv().unwrap();
        });
    });
    ready.recv_timeout(Duration::from_secs(5)).unwrap();
    let (reading, read_started) = mpsc::channel();
    let (read, read_done) = mpsc::channel();
    let reader = thread::spawn(move || {
        reading.send(()).unwrap();
        read.send(region.create_reader().read_all().to_vec())
            .unwrap();
    });
    let (flushing, flush_started) = mpsc::channel();
    let (flushed, flush_done) = mpsc::channel();
    let flusher = thread::spawn(move || {
        flushing.send(()).unwrap();
        flushed.send(db.flush()).unwrap();
    });
    read_started.recv_timeout(Duration::from_secs(5)).unwrap();
    flush_started.recv_timeout(Duration::from_secs(5)).unwrap();
    let read_blocked = read_done.recv_timeout(Duration::from_millis(30)).is_err();
    let flush_blocked = flush_done.recv_timeout(Duration::from_millis(30)).is_err();
    release.send(()).unwrap();
    writer.join().unwrap();
    reader.join().unwrap();
    flusher.join().unwrap();
    assert!(read_blocked, "readers must not observe a partial batch");
    assert!(
        flush_blocked,
        "flush must wait until the batch marks its writes dirty"
    );
    assert_eq!(read_done.recv().unwrap(), [7; 8]);
    assert_eq!(flush_done.recv().unwrap()?, 1);
    Ok(())
}
