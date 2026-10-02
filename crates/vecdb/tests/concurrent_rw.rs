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
    AnyStoredVec, AnyVec, BytesVec, ImportableVec, Result, StoredVec, Version, WritableVec,
};

fn setup_test_db() -> Result<(Database, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db = Database::open(temp_dir.path())?;
    Ok((db, temp_dir))
}

/// A newly published length must have readable data behind it for every reader.
#[test]
fn test_memory_ordering_len_vs_data() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let mut writer: BytesVec<usize, u64> = BytesVec::forced_import(&db, "test_vec", Version::ONE)?;
    let barrier = Arc::new(Barrier::new(9));
    let stop = Arc::new(AtomicBool::new(false));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let reader = writer.read_only_clone();
            let barrier = barrier.clone();
            let stop = stop.clone();
            thread::spawn(move || {
                assert!(reader.is_empty());
                barrier.wait();
                while !stop.load(Ordering::Relaxed) {
                    let len = reader.len();
                    if len > 0 {
                        let r = reader.reader();
                        for i in [0, len / 2, len - 1] {
                            assert_eq!(r.get(i), i as u64);
                        }
                    }
                }
                let r = reader.reader();
                assert_eq!(r.stored_len(), 1000);
                assert_eq!(r.get(999), 999);
            })
        })
        .collect();
    barrier.wait();
    for batch in 0..100 {
        for i in 0..10u64 {
            writer.push(batch * 10 + i);
        }
        writer.write()?;
        thread::yield_now();
    }
    thread::sleep(Duration::from_millis(1));
    stop.store(true, Ordering::Relaxed);
    for handle in handles {
        handle.join().unwrap();
    }
    assert_eq!(writer.len(), 1000);
    Ok(())
}
