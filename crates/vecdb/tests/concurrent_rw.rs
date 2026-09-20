//! Concurrent reads during writes and batched syncs.
//! Data must reach the mmap before the stored length is published.
//! If a reader sees a new stored_len, the corresponding data MUST be readable.

use std::{
    sync::{
        Arc, Barrier,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use rawdb::Database;
use tempfile::TempDir;
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, ImportableVec, ReadableVec, Result, StoredVec, Version,
    WritableVec,
};

#[cfg(feature = "pco")]
use vecdb::PcoVec;

fn setup_test_db() -> Result<(Database, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db = Database::open(temp_dir.path())?;
    Ok((db, temp_dir))
}

/// Test that a cloned reader can see data after writer calls write() but before flush()
#[test]
fn test_reader_sees_written_data_without_flush() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let version = Version::ONE;

    // Create writer vec and write initial data
    let mut writer: BytesVec<usize, u64> = BytesVec::forced_import(&db, "test_vec", version)?;

    for i in 0..100u64 {
        writer.push(i);
    }
    writer.write()?;
    // Note: NOT calling flush() here - data is in mmap but not synced to disk

    // Create VecReader (simulates what Query does)
    let r = writer.reader();

    // Reader should see the stored data via shared mmap
    assert_eq!(r.len(), 100);
    assert_eq!(r.get(0), 0);
    assert_eq!(r.get(50), 50);
    assert_eq!(r.get(99), 99);

    Ok(())
}

/// Test that reader sees new data written after clone, once write() is called
#[test]
fn test_reader_sees_new_data_after_write() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let version = Version::ONE;

    // Create and initialize writer
    let mut writer: BytesVec<usize, u64> = BytesVec::forced_import(&db, "test_vec", version)?;

    for i in 0..50u64 {
        writer.push(i);
    }
    writer.write()?;

    // Create read-only clone sharing SharedLen
    let reader = writer.read_only_clone();
    assert_eq!(reader.len(), 50);

    // Writer adds more data
    for i in 50..100u64 {
        writer.push(i);
    }

    // Pushed data is visible only to the writer until write().
    assert_eq!(writer.len(), 100);
    assert_eq!(writer.pushed_len(), 50);
    assert_eq!(reader.len(), 50);
    {
        let r = reader.reader();
        assert_eq!(r.get(49), 49);
        assert_eq!(r.try_get(50), None);
    }

    // Writer calls write() - this updates stored_len (shared) and writes to mmap
    writer.write()?;

    // Now reader should see the new stored_len
    assert_eq!(reader.len(), 100);

    // And read-only clone can create a VecReader for O(1) point reads
    let r = reader.reader();
    assert_eq!(r.get(99), 99);
    assert_eq!(r.get(75), 75);

    Ok(())
}

/// Test concurrent read while write is happening
#[test]
fn test_concurrent_read_during_write() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let version = Version::ONE;

    let mut writer: BytesVec<usize, u64> = BytesVec::forced_import(&db, "test_vec", version)?;

    // Write initial batch
    for i in 0..1000u64 {
        writer.push(i);
    }
    writer.write()?;

    let reader = writer.read_only_clone();
    let barrier = Arc::new(Barrier::new(2));

    let reader_barrier = barrier.clone();
    let reader_handle = thread::spawn(move || {
        // Wait for writer to start
        reader_barrier.wait();

        // Continuously read while writer is working
        for _ in 0..100 {
            let len = reader.len();
            if len > 0 {
                let r = reader.reader();
                // Read some values - should never panic or return garbage
                for i in 0..len.min(100) {
                    assert_eq!(r.try_get(i), Some(i as u64));
                }
            }
            thread::sleep(Duration::from_micros(100));
        }
    });

    // Signal reader to start, then write more data
    barrier.wait();

    for batch in 0..10 {
        for i in 0..100u64 {
            writer.push(1000 + batch * 100 + i);
        }
        writer.write()?;
        thread::sleep(Duration::from_micros(50));
    }

    reader_handle.join().unwrap();

    // Final state check
    assert_eq!(writer.len(), 2000);

    Ok(())
}

/// Test that multiple vecs can be written without flush, then flushed together
#[test]
fn test_batched_writes_single_flush() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let version = Version::ONE;

    let mut vec1: BytesVec<usize, u64> = BytesVec::forced_import(&db, "vec1", version)?;
    let mut vec2: BytesVec<usize, u64> = BytesVec::forced_import(&db, "vec2", version)?;
    let mut vec3: BytesVec<usize, u64> = BytesVec::forced_import(&db, "vec3", version)?;

    // Write to all vecs without flushing
    for i in 0..100u64 {
        vec1.push(i);
        vec2.push(i * 2);
        vec3.push(i * 3);
    }

    // Write all (to mmap) without flush
    vec1.write()?;
    vec2.write()?;
    vec3.write()?;

    // Create VecReaders
    let r1 = vec1.reader();
    let r2 = vec2.reader();
    let r3 = vec3.reader();

    // All readers should see the data
    assert_eq!(r1.len(), 100);
    assert_eq!(r2.len(), 100);
    assert_eq!(r3.len(), 100);

    assert_eq!(r1.get(50), 50);
    assert_eq!(r2.get(50), 100);
    assert_eq!(r3.get(50), 150);

    // Flush while readers are still alive - no deadlock since
    // dirty_range is in a separate Mutex from region metadata
    db.flush()?;

    // Data should still be readable after flush
    drop(r1);
    drop(r2);
    drop(r3);

    let r1 = vec1.reader();
    assert_eq!(r1.get(99), 99);

    Ok(())
}

/// Test with PcoVec (compressed) to ensure it also works
#[test]
#[cfg(feature = "pco")]
fn test_pco_concurrent_read_write() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let version = Version::ONE;

    let mut writer: PcoVec<usize, u64> = PcoVec::forced_import(&db, "pco_vec", version)?;

    for i in 0..500u64 {
        writer.push(i);
    }
    writer.write()?;

    let reader = writer.read_only_clone();

    // Add more data
    for i in 500..1000u64 {
        writer.push(i);
    }
    writer.write()?;

    // Reader should see all data
    assert_eq!(reader.len(), 1000);

    assert_eq!(reader.collect_range(0, 1), vec![0]);
    assert_eq!(reader.collect_range(500, 501), vec![500]);
    assert_eq!(reader.collect_range(999, 1000), vec![999]);

    Ok(())
}

/// A newly published length must have readable data behind it.
#[test]
fn test_memory_ordering_len_vs_data() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let version = Version::ONE;

    let mut writer: BytesVec<usize, u64> = BytesVec::forced_import(&db, "test_vec", version)?;

    // Write initial data
    for i in 0..100u64 {
        writer.push(i);
    }
    writer.write()?;

    let reader = writer.read_only_clone();

    let barrier = Arc::new(Barrier::new(2));
    let stop = Arc::new(AtomicBool::new(false));

    let reader_barrier = barrier.clone();
    let stop_clone = stop.clone();

    // Reader thread: continuously check that if we see a length, we can read that data
    let reader_handle = thread::spawn(move || {
        let mut reads = 0;
        // Wait for writer to be ready
        reader_barrier.wait();

        while !stop_clone.load(Ordering::Relaxed) {
            let len = reader.len();
            if len > 0 {
                // CRITICAL: If we see len = N, we MUST be able to read index N-1
                let last_idx = len - 1;
                let r = reader.reader();
                assert_eq!(r.get(last_idx), last_idx as u64);
                reads += 1;
            }
            // No sleep - tight loop to maximize chance of catching races
        }
        reads
    });

    // Synchronize start with reader
    barrier.wait();

    // Writer thread: keep adding data
    for batch in 0..100 {
        for i in 0..10u64 {
            let val = 100 + batch * 10 + i;
            writer.push(val);
        }
        writer.write()?;
        // Small yield to give reader a chance to run
        thread::yield_now();
    }

    // Let reader run a bit more after writer is done
    thread::sleep(Duration::from_millis(1));

    stop.store(true, Ordering::Relaxed);
    let read_count = reader_handle.join().unwrap();
    assert!(read_count > 0, "Should have completed at least some reads");

    // Verify final state
    assert_eq!(writer.len(), 1100);

    Ok(())
}

/// Test that reader always sees consistent length and can read up to that length
#[test]
fn test_length_data_consistency_stress() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let version = Version::ONE;

    let mut writer: BytesVec<usize, u64> = BytesVec::forced_import(&db, "test_vec", version)?;

    let reader = writer.read_only_clone();

    let stop = Arc::new(AtomicBool::new(false));

    let stop_clone = stop.clone();

    // Reader aggressively checks consistency
    let reader_handle = thread::spawn(move || {
        let mut max_len = 0;
        for _ in 0..1000 {
            if stop_clone.load(Ordering::Relaxed) {
                break;
            }

            let len = reader.len();
            max_len = max_len.max(len);

            if len > 0 {
                // Create fresh VecReader each time to pick up new stored data
                let r = reader.reader();

                // Check first, last, and a few sample indices
                for i in [0, len - 1, len / 2] {
                    assert_eq!(r.get(i), i as u64);
                }
            }
            thread::sleep(Duration::from_micros(10));
        }
        max_len
    });

    // Writer rapidly adds data
    for i in 0..500u64 {
        writer.push(i);
        if i % 10 == 0 {
            writer.write()?;
        }
    }
    writer.write()?;

    // Let reader catch up
    thread::sleep(Duration::from_millis(10));
    stop.store(true, Ordering::Relaxed);
    let max_len = reader_handle.join().unwrap();
    assert!(max_len > 0, "Reader should have seen some data");

    Ok(())
}

/// Stress test with many concurrent readers and one writer
#[test]
fn test_many_readers_one_writer() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let version = Version::ONE;

    let mut writer: BytesVec<usize, u64> = BytesVec::forced_import(&db, "test_vec", version)?;

    // Initial data
    for i in 0..100u64 {
        writer.push(i);
    }
    writer.write()?;

    let num_readers = 8;
    let barrier = Arc::new(Barrier::new(num_readers + 1));

    let handles: Vec<_> = (0..num_readers)
        .map(|_| {
            let reader = writer.read_only_clone();
            let b = barrier.clone();
            thread::spawn(move || {
                b.wait();
                for _ in 0..50 {
                    let r = reader.reader();
                    let len = r.len();
                    // Verify data integrity
                    for i in 0..len.min(100) {
                        let val = r.get(i);
                        assert_eq!(val, i as u64);
                    }
                    thread::sleep(Duration::from_micros(10));
                }
            })
        })
        .collect();

    barrier.wait();

    // Writer keeps adding data
    for batch in 0..20 {
        for i in 0..50u64 {
            writer.push(100 + batch * 50 + i);
        }
        writer.write()?;
        thread::sleep(Duration::from_micros(100));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(writer.len(), 1100);

    Ok(())
}
