//! Tests for concurrent read/write behavior with batched syncs.
//!
//! These tests verify that:
//! 1. A reader clone can see data written by writer after write() (even without sync)
//! 2. Batched writes followed by a single sync are safe for concurrent readers
//! 3. The SharedLen atomic is properly visible across clones
//! 4. Data written to mmap is visible to readers BEFORE stored_len is updated
//!    (critical for memory ordering correctness)
//!
//! IMPORTANT: These tests verify that the write ordering is correct:
//!   1. Data must be written to mmap
//!   2. Memory barrier (implicit in SeqCst atomic)
//!   3. stored_len is updated
//!
//! If a reader sees a new stored_len, the corresponding data MUST be readable.

use std::{
    sync::{
        Arc, Barrier,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

use rawdb::Database;
use tempfile::TempDir;
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, ImportableVec, ReadableVec, StoredVec, Version, WritableVec,
};

#[cfg(feature = "pco")]
use vecdb::PcoVec;

fn setup_test_db() -> vecdb::Result<(Database, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db = Database::open(temp_dir.path())?;
    Ok((db, temp_dir))
}

/// Test that a cloned reader can see data after writer calls write() but before flush()
#[test]
fn test_reader_sees_written_data_without_flush() -> vecdb::Result<()> {
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
fn test_reader_sees_new_data_after_write() -> vecdb::Result<()> {
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

    // Reader still sees old stored_len (pushed is not shared)
    assert_eq!(reader.len(), 50);

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
fn test_concurrent_read_during_write() -> vecdb::Result<()> {
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
    let reader_handle = thread::spawn(move || -> vecdb::Result<()> {
        // Wait for writer to start
        reader_barrier.wait();

        // Continuously read while writer is working
        for _ in 0..100 {
            let len = reader.len();
            if len > 0 {
                let r = reader.reader();
                // Read some values - should never panic or return garbage
                for i in 0..len.min(100) {
                    let val = r.try_get(i);
                    assert!(val.is_some(), "Expected value at index {}", i);
                    assert_eq!(val.unwrap(), i as u64);
                }
            }
            thread::sleep(Duration::from_micros(100));
        }
        Ok(())
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

    reader_handle.join().unwrap()?;

    // Final state check
    assert_eq!(writer.len(), 2000);

    Ok(())
}

/// Test that multiple vecs can be written without flush, then flushed together
#[test]
fn test_batched_writes_single_flush() -> vecdb::Result<()> {
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
fn test_pco_concurrent_read_write() -> vecdb::Result<()> {
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

/// Test that reader doesn't see writer's uncommitted pushed data
#[test]
fn test_reader_isolation_from_pushed() -> vecdb::Result<()> {
    let (db, _temp) = setup_test_db()?;
    let version = Version::ONE;

    let mut writer: BytesVec<usize, u64> = BytesVec::forced_import(&db, "test_vec", version)?;

    for i in 0..50u64 {
        writer.push(i);
    }
    writer.write()?;

    // Read-only clone
    let reader = writer.read_only_clone();

    // Writer pushes more but doesn't write
    for i in 50..100u64 {
        writer.push(i);
    }

    // Writer sees pushed data
    assert_eq!(writer.len(), 100);
    assert_eq!(writer.pushed_len(), 50);

    // Read-only clone doesn't see writer's pushed
    assert_eq!(reader.len(), 50);

    // Reader can't access indices 50-99
    let r = reader.reader();
    assert_eq!(r.get(49), 49);
    assert_eq!(r.try_get(50), None);

    Ok(())
}

/// CRITICAL TEST: Verify that when reader sees updated stored_len, the data is readable.
/// This tests the memory ordering invariant: mmap write must happen-before stored_len update.
#[test]
fn test_memory_ordering_len_vs_data() -> vecdb::Result<()> {
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
    let errors = Arc::new(AtomicUsize::new(0));
    let reads = Arc::new(AtomicUsize::new(0));

    let reader_barrier = barrier.clone();
    let stop_clone = stop.clone();
    let errors_clone = errors.clone();
    let reads_clone = reads.clone();

    // Reader thread: continuously check that if we see a length, we can read that data
    let reader_handle = thread::spawn(move || {
        // Wait for writer to be ready
        reader_barrier.wait();

        while !stop_clone.load(Ordering::Relaxed) {
            let len = reader.len();
            if len > 0 {
                // CRITICAL: If we see len = N, we MUST be able to read index N-1
                let last_idx = len - 1;
                let r = reader.reader();
                let val = r.get(last_idx);
                if val != last_idx as u64 {
                    eprintln!(
                        "ERROR: Read wrong value at {}: expected {}, got {}",
                        last_idx, last_idx, val
                    );
                    errors_clone.fetch_add(1, Ordering::Relaxed);
                }
                reads_clone.fetch_add(1, Ordering::Relaxed);
            }
            // No sleep - tight loop to maximize chance of catching races
        }
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
    reader_handle.join().unwrap();

    let error_count = errors.load(Ordering::Relaxed);
    let read_count = reads.load(Ordering::Relaxed);

    println!("Completed {} reads with {} errors", read_count, error_count);

    assert_eq!(error_count, 0, "Memory ordering violation detected!");
    assert!(read_count > 0, "Should have completed at least some reads");

    // Verify final state
    assert_eq!(writer.len(), 1100);

    Ok(())
}

/// Test that reader always sees consistent length and can read up to that length
#[test]
fn test_length_data_consistency_stress() -> vecdb::Result<()> {
    let (db, _temp) = setup_test_db()?;
    let version = Version::ONE;

    let mut writer: BytesVec<usize, u64> = BytesVec::forced_import(&db, "test_vec", version)?;

    let reader = writer.read_only_clone();

    let stop = Arc::new(AtomicBool::new(false));
    let max_len_seen = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(AtomicUsize::new(0));

    let stop_clone = stop.clone();
    let max_len_clone = max_len_seen.clone();
    let errors_clone = errors.clone();

    // Reader aggressively checks consistency
    let reader_handle = thread::spawn(move || {
        for _ in 0..1000 {
            if stop_clone.load(Ordering::Relaxed) {
                break;
            }

            let len = reader.len();
            max_len_clone.fetch_max(len, Ordering::Relaxed);

            if len > 0 {
                // Create fresh VecReader each time to pick up new stored data
                let r = reader.reader();

                // Check first, last, and a few sample indices
                let indices_to_check = [0, len.saturating_sub(1), len / 2];

                for &i in &indices_to_check {
                    if i >= len {
                        continue;
                    }
                    let val = r.get(i);
                    if val != i as u64 {
                        errors_clone.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            thread::sleep(Duration::from_micros(10));
        }
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
    reader_handle.join().unwrap();

    let error_count = errors.load(Ordering::Relaxed);
    assert_eq!(error_count, 0, "Consistency violation detected!");

    // Reader should have seen at least some of the data
    let max_len = max_len_seen.load(Ordering::Relaxed);
    println!("Reader saw max len: {}", max_len);
    assert!(max_len > 0, "Reader should have seen some data");

    Ok(())
}

/// Stress test with many concurrent readers and one writer
#[test]
fn test_many_readers_one_writer() -> vecdb::Result<()> {
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
            thread::spawn(move || -> vecdb::Result<()> {
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
                Ok(())
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
        handle.join().unwrap()?;
    }

    assert_eq!(writer.len(), 1100);

    Ok(())
}

/// Long-running realistic stress test that simulates real-world usage patterns.
/// This test:
/// - Runs for several seconds
/// - Has multiple vecs being written concurrently (like brk_computer)
/// - Has readers continuously verifying data integrity
/// - Uses batched writes without intermediate flushes
/// - Only flushes at the end of each "block" (simulating block processing)
///
/// Run with: cargo test --features pco test_realworld_stress -- --ignored --nocapture
#[test]
#[ignore] // Run manually: takes ~10 seconds
fn test_realworld_stress() -> vecdb::Result<()> {
    use std::time::Instant;

    let (db, _temp) = setup_test_db()?;
    let version = Version::ONE;

    // Create multiple vecs (simulating different metrics in brk_computer)
    let mut vec_a: BytesVec<usize, u64> = BytesVec::forced_import(&db, "metric_a", version)?;
    let mut vec_b: BytesVec<usize, u64> = BytesVec::forced_import(&db, "metric_b", version)?;
    let mut vec_c: BytesVec<usize, u64> = BytesVec::forced_import(&db, "metric_c", version)?;

    // Write some initial data
    for i in 0..1000u64 {
        vec_a.push(i);
        vec_b.push(i * 2);
        vec_c.push(i * 3);
    }
    vec_a.write()?;
    vec_b.write()?;
    vec_c.write()?;
    db.flush()?;

    // Create read-only clones (simulating web server Query clones)
    let reader_a = vec_a.read_only_clone();
    let reader_b = vec_b.read_only_clone();
    let reader_c = vec_c.read_only_clone();

    let stop = Arc::new(AtomicBool::new(false));
    let total_reads = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(AtomicUsize::new(0));

    // Spawn multiple reader threads (simulating concurrent API requests)
    let num_readers = 4;
    let reader_handles: Vec<_> = (0..num_readers)
        .map(|reader_id| {
            let r_a = reader_a.clone();
            let r_b = reader_b.clone();
            let r_c = reader_c.clone();
            let stop = stop.clone();
            let reads = total_reads.clone();
            let errs = errors.clone();

            thread::spawn(move || {
                let mut local_reads = 0u64;
                let mut local_errors = 0u64;

                while !stop.load(Ordering::Relaxed) {
                    // Read from all three vecs
                    let len_a = r_a.len();
                    let len_b = r_b.len();
                    let len_c = r_c.len();

                    // They should all have the same length (written together)
                    // Allow for small differences during concurrent writes
                    let max_len = len_a.max(len_b).max(len_c);
                    let min_len = len_a.min(len_b).min(len_c);
                    if max_len - min_len > 100 {
                        eprintln!(
                            "Reader {}: Large length discrepancy: a={}, b={}, c={}",
                            reader_id, len_a, len_b, len_c
                        );
                        local_errors += 1;
                    }

                    // Verify data at various positions
                    if min_len > 0 {
                        let ra = r_a.reader();
                        let rb = r_b.reader();
                        let rc = r_c.reader();

                        // Check first element
                        if ra.get(0) != 0 {
                            local_errors += 1;
                        }

                        // Check last safe element
                        let safe_idx = min_len.saturating_sub(1);
                        let va = ra.get(safe_idx);
                        if va != safe_idx as u64 {
                            eprintln!(
                                "Reader {}: vec_a[{}] = {} (expected {})",
                                reader_id, safe_idx, va, safe_idx
                            );
                            local_errors += 1;
                        }
                        if rb.get(safe_idx) != (safe_idx as u64) * 2 {
                            local_errors += 1;
                        }
                        if rc.get(safe_idx) != (safe_idx as u64) * 3 {
                            local_errors += 1;
                        }

                        // Check a middle element
                        let mid_idx = min_len / 2;
                        if ra.get(mid_idx) != mid_idx as u64 {
                            local_errors += 1;
                        }

                        local_reads += 1;
                    }

                    // Simulate realistic request rate
                    thread::sleep(Duration::from_micros(100));
                }

                reads.fetch_add(local_reads as usize, Ordering::Relaxed);
                errs.fetch_add(local_errors as usize, Ordering::Relaxed);
            })
        })
        .collect();

    // Writer thread: simulate processing blocks
    let start = Instant::now();
    let num_blocks = 100;
    let values_per_block = 50;
    let mut current_idx = 1000u64;

    println!(
        "Starting {} blocks of {} values each...",
        num_blocks, values_per_block
    );

    for block in 0..num_blocks {
        // Simulate processing a block - write to multiple vecs
        for _ in 0..values_per_block {
            vec_a.push(current_idx);
            vec_b.push(current_idx * 2);
            vec_c.push(current_idx * 3);
            current_idx += 1;
        }

        // Batched write: write all vecs without flushing each one
        vec_a.write()?;
        vec_b.write()?;
        vec_c.write()?;

        // Single flush at end of block (this is the optimization we're testing)
        db.flush()?;

        // Simulate some processing time between blocks
        if block % 10 == 0 {
            println!("Processed block {}/{}", block + 1, num_blocks);
        }
        thread::sleep(Duration::from_millis(10));
    }

    let write_duration = start.elapsed();

    // Stop readers and wait for them
    stop.store(true, Ordering::Relaxed);
    thread::sleep(Duration::from_millis(50)); // Give readers time to notice

    for handle in reader_handles {
        handle.join().unwrap();
    }

    let final_reads = total_reads.load(Ordering::Relaxed);
    let final_errors = errors.load(Ordering::Relaxed);

    println!("\n=== Results ===");
    println!("Write duration: {:?}", write_duration);
    println!(
        "Final vec lengths: a={}, b={}, c={}",
        vec_a.len(),
        vec_b.len(),
        vec_c.len()
    );
    println!("Total reader verifications: {}", final_reads);
    println!("Errors detected: {}", final_errors);

    // Verify final state
    let expected_len = 1000 + (num_blocks * values_per_block) as usize;
    assert_eq!(vec_a.len(), expected_len);
    assert_eq!(vec_b.len(), expected_len);
    assert_eq!(vec_c.len(), expected_len);
    assert_eq!(final_errors, 0, "Data integrity errors detected!");
    assert!(
        final_reads > 100,
        "Should have completed many read verifications"
    );

    // Verify all data is correct after everything is done
    println!("Verifying final data integrity...");
    let reader_a_ref = vec_a.reader();
    let reader_b_ref = vec_b.reader();
    let reader_c_ref = vec_c.reader();

    for i in 0..expected_len {
        let a = reader_a_ref.get(i);
        let b = reader_b_ref.get(i);
        let c = reader_c_ref.get(i);

        assert_eq!(a, i as u64, "vec_a[{}] incorrect", i);
        assert_eq!(b, (i as u64) * 2, "vec_b[{}] incorrect", i);
        assert_eq!(c, (i as u64) * 3, "vec_c[{}] incorrect", i);
    }

    println!("All {} values verified correctly!", expected_len);

    Ok(())
}

/// Even longer stress test with more aggressive concurrent access
/// Run with: cargo test --features pco test_extended_stress -- --ignored --nocapture
#[test]
#[ignore] // Run manually: takes ~30 seconds
fn test_extended_stress() -> vecdb::Result<()> {
    use std::time::Instant;

    let (db, _temp) = setup_test_db()?;
    let version = Version::ONE;

    // Use compressed vecs for more realistic scenario
    #[cfg(feature = "pco")]
    let mut writer: PcoVec<usize, u64> = PcoVec::forced_import(&db, "stress_vec", version)?;
    #[cfg(not(feature = "pco"))]
    let mut writer: BytesVec<usize, u64> = BytesVec::forced_import(&db, "stress_vec", version)?;

    let stop = Arc::new(AtomicBool::new(false));
    let writes_completed = Arc::new(AtomicUsize::new(0));
    let reads_completed = Arc::new(AtomicUsize::new(0));
    let read_errors = Arc::new(AtomicUsize::new(0));

    // Single reader to verify the fix works
    let num_readers = 1;
    let reader_handles: Vec<_> = (0..num_readers)
        .map(|_| {
            let reader = writer.read_only_clone();
            let stop = stop.clone();
            let reads = reads_completed.clone();
            let errs = read_errors.clone();

            thread::spawn(move || {
                let mut local_reads = 0usize;
                let mut local_errors = 0usize;

                while !stop.load(Ordering::Relaxed) {
                    let len = reader.len();
                    if len > 0 {
                        // Check last element - most likely to catch races
                        let idx = len - 1;
                        if let Some(v) = reader.collect_one(idx) {
                            if v != idx as u64 {
                                local_errors += 1;
                            }
                            local_reads += 1;
                        } else {
                            local_errors += 1;
                        }

                        // Also check a random-ish index
                        let random_idx = (len * 7) / 11; // Pseudo-random
                        if random_idx < len
                            && let Some(v) = reader.collect_one(random_idx)
                        {
                            if v != random_idx as u64 {
                                local_errors += 1;
                            }
                            local_reads += 1;
                        }
                    }
                    // Small sleep between reads
                    thread::sleep(Duration::from_micros(100));
                }

                reads.fetch_add(local_reads, Ordering::Relaxed);
                errs.fetch_add(local_errors, Ordering::Relaxed);
            })
        })
        .collect();

    // Writer: infrequent writes (simulates real brk_computer pattern)
    let start = Instant::now();
    let target_duration = Duration::from_secs(5);
    let mut current_idx = 0u64;
    let mut batches = 0usize;

    println!("Running stress test for {:?}...", target_duration);

    while start.elapsed() < target_duration {
        // Large batch size per write (like processing a block)
        let batch_size = 100;

        for _ in 0..batch_size {
            writer.push(current_idx);
            current_idx += 1;
        }

        writer.write()?;
        writes_completed.fetch_add(1, Ordering::Relaxed);
        batches += 1;

        if batches.is_multiple_of(10) {
            println!("Written {} batches, {} values", batches, current_idx);
        }

        // Sleep between writes - simulates ~10 minute block interval scaled down
        // 100ms sleep = 50 writes over 5 seconds
        thread::sleep(Duration::from_millis(100));

        // Flush every 10 batches
        if batches.is_multiple_of(10) {
            db.flush()?;
        }
    }

    // Final flush
    db.flush()?;

    let write_duration = start.elapsed();

    // Stop readers
    stop.store(true, Ordering::Relaxed);
    thread::sleep(Duration::from_millis(100));

    for handle in reader_handles {
        handle.join().unwrap();
    }

    let final_writes = writes_completed.load(Ordering::Relaxed);
    let final_reads = reads_completed.load(Ordering::Relaxed);
    let final_errors = read_errors.load(Ordering::Relaxed);

    println!("\n=== Extended Stress Test Results ===");
    println!("Duration: {:?}", write_duration);
    println!("Total values written: {}", current_idx);
    println!("Write batches: {}", final_writes);
    println!("Read verifications: {}", final_reads);
    println!("Errors: {}", final_errors);
    println!(
        "Reads per second: {:.0}",
        final_reads as f64 / write_duration.as_secs_f64()
    );

    assert_eq!(writer.len(), current_idx as usize);
    assert_eq!(final_errors, 0, "Data integrity errors detected!");
    assert!(final_reads > 1000, "Should have many read verifications");

    // Spot-check final data
    println!("Spot-checking final data...");
    for i in [
        0,
        100,
        1000,
        current_idx as usize / 2,
        current_idx as usize - 1,
    ] {
        if i < writer.len() {
            let v = writer.collect_one(i).unwrap();
            assert_eq!(v, i as u64, "Value at {} incorrect", i);
        }
    }
    println!("Spot-check passed!");

    Ok(())
}

/// BytesVec version of extended stress test - runs without compression overhead
/// This allows for a true tight loop test without reader starvation issues.
///
/// Run with: cargo test test_extended_stress_bytes -- --ignored --nocapture
#[test]
#[ignore] // Run manually
fn test_extended_stress_bytes() -> vecdb::Result<()> {
    use std::time::Instant;

    let (db, _temp) = setup_test_db()?;
    let version = Version::ONE;

    let mut writer: BytesVec<usize, u64> =
        BytesVec::forced_import(&db, "stress_vec_bytes", version)?;

    let stop = Arc::new(AtomicBool::new(false));
    let writes_completed = Arc::new(AtomicUsize::new(0));
    let reads_completed = Arc::new(AtomicUsize::new(0));
    let read_errors = Arc::new(AtomicUsize::new(0));

    // More aggressive readers - BytesVec is fast enough for tight loops
    let num_readers = 8;
    let reader_handles: Vec<_> = (0..num_readers)
        .map(|_| {
            let reader = writer.read_only_clone();
            let stop = stop.clone();
            let reads = reads_completed.clone();
            let errs = read_errors.clone();

            thread::spawn(move || {
                let mut local_reads = 0usize;
                let mut local_errors = 0usize;

                while !stop.load(Ordering::Relaxed) {
                    let r = reader.reader();
                    let len = r.len();
                    if len > 0 {
                        // Check last element - most likely to catch races
                        let idx = len - 1;
                        let v = r.get(idx);
                        if v != idx as u64 {
                            local_errors += 1;
                        }
                        local_reads += 1;

                        // Also check a random-ish index
                        let random_idx = (len * 7) / 11;
                        if random_idx < len {
                            let v = r.get(random_idx);
                            if v != random_idx as u64 {
                                local_errors += 1;
                            }
                            local_reads += 1;
                        }
                    }
                    // Very tight loop for BytesVec - no sleep needed
                }

                reads.fetch_add(local_reads, Ordering::Relaxed);
                errs.fetch_add(local_errors, Ordering::Relaxed);
            })
        })
        .collect();

    // Writer: rapid writes with varying batch sizes
    let start = Instant::now();
    let target_duration = Duration::from_secs(5);
    let mut current_idx = 0u64;
    let mut batches = 0usize;

    println!("Running BytesVec stress test for {:?}...", target_duration);

    while start.elapsed() < target_duration {
        // Varying batch sizes
        let batch_size = 10 + (batches % 50);

        for _ in 0..batch_size {
            writer.push(current_idx);
            current_idx += 1;
        }

        writer.write()?;
        writes_completed.fetch_add(1, Ordering::Relaxed);
        batches += 1;

        // Occasionally flush (simulating block boundaries)
        if batches.is_multiple_of(10) {
            db.flush()?;
        }
    }

    // Final flush
    db.flush()?;

    let write_duration = start.elapsed();

    // Stop readers
    stop.store(true, Ordering::Relaxed);
    thread::sleep(Duration::from_millis(100));

    for handle in reader_handles {
        handle.join().unwrap();
    }

    let final_writes = writes_completed.load(Ordering::Relaxed);
    let final_reads = reads_completed.load(Ordering::Relaxed);
    let final_errors = read_errors.load(Ordering::Relaxed);

    println!("\n=== BytesVec Extended Stress Test Results ===");
    println!("Duration: {:?}", write_duration);
    println!("Total values written: {}", current_idx);
    println!("Write batches: {}", final_writes);
    println!("Read verifications: {}", final_reads);
    println!("Errors: {}", final_errors);
    println!(
        "Reads per second: {:.0}",
        final_reads as f64 / write_duration.as_secs_f64()
    );

    assert_eq!(writer.len(), current_idx as usize);
    assert_eq!(final_errors, 0, "Data integrity errors detected!");
    assert!(
        final_reads > 10000,
        "Should have many read verifications with tight loop"
    );

    // Spot-check final data
    println!("Spot-checking final data...");
    let reader_ref = writer.reader();
    for i in [
        0,
        100,
        1000,
        current_idx as usize / 2,
        current_idx as usize - 1,
    ] {
        if i < writer.len() {
            let v = reader_ref.get(i);
            assert_eq!(v, i as u64, "Value at {} incorrect", i);
        }
    }
    println!("Spot-check passed!");

    Ok(())
}
