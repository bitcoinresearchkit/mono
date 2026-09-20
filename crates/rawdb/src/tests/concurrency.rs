use std::{
    sync::{Arc, Barrier, mpsc},
    thread,
    time::Duration,
};

use tempfile::TempDir;

use crate::{Database, Result};

use super::setup_test_db;

#[test]
fn readers_exclude_overwrites_but_allow_other_regions_and_flush() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let region = db.create_region_if_needed("read")?;
    let other = db.create_region_if_needed("other")?;
    region.write(b"original")?;
    let reader = region.create_reader();
    let (started, ready) = mpsc::channel();
    let (done, completed) = mpsc::channel();
    let writer = thread::spawn(move || {
        started.send(()).unwrap();
        region.write_at(b"replaced", 0).unwrap();
        done.send(()).unwrap();
    });
    ready.recv_timeout(Duration::from_secs(5)).unwrap();
    let blocked = completed.recv_timeout(Duration::from_millis(30)).is_err();
    other.write(b"independent")?;
    let (flushed, result) = mpsc::channel();
    let flushing_db = db.clone();
    let flusher = thread::spawn(move || flushed.send(flushing_db.flush()).unwrap());
    let flush_result = result.recv_timeout(Duration::from_secs(5));
    assert_eq!(reader.read_all(), b"original");
    assert_eq!(reader.read_from(0), b"original");
    drop(reader);
    writer.join().unwrap();
    flusher.join().unwrap();
    assert!(blocked, "a live reader must exclude writes to its region");
    flush_result.expect("a waiting writer must not block flush")?;
    assert_eq!(
        db.get_region("read").unwrap().create_reader().read_all(),
        b"replaced"
    );
    Ok(())
}

#[test]
fn concurrent_appends_to_one_region_preserve_every_record() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let region = db.create_region_if_needed("shared")?;
    let barrier = Arc::new(Barrier::new(8));
    let threads: Vec<_> = (0..8u8)
        .map(|id| {
            let region = region.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                for _ in 0..256 {
                    region.write(&[id; 16]).unwrap();
                }
            })
        })
        .collect();
    for task in threads {
        task.join().unwrap();
    }
    let reader = region.create_reader();
    assert_eq!(reader.len(), 8 * 256 * 16);
    let mut counts = [0; 8];
    for record in reader.read_all().as_chunks::<16>().0 {
        assert!(record.iter().all(|&byte| byte == record[0]));
        counts[usize::from(record[0])] += 1;
    }
    assert_eq!(counts, [256; 8]);
    Ok(())
}

#[test]
fn concurrent_creation_crosses_mapping_capacity_safely() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let threads: Vec<_> = (0..8)
        .map(|group| {
            let db = db.clone();
            thread::spawn(move || {
                for item in 0..64 {
                    let name = format!("{group}/{item}");
                    db.create_region_if_needed(&name)
                        .unwrap()
                        .write(name.as_bytes())
                        .unwrap();
                }
            })
        })
        .collect();
    for task in threads {
        task.join().unwrap();
    }
    db.flush()?;
    drop(db);
    let db = Database::open(dir.path())?;
    assert_eq!(db.regions().len(), 512);
    for group in 0..8 {
        for item in 0..64 {
            let name = format!("{group}/{item}");
            assert_eq!(
                db.get_region(&name).unwrap().create_reader().read_all(),
                name.as_bytes()
            );
        }
    }
    Ok(())
}

#[test]
fn concurrent_last_owner_drops_join_deferred_work() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let region = db.create_region_if_needed("value")?;
    region.write(b"persisted")?;
    db.run_bg(|db| db.compact_deferred_default());
    let second = db.clone();
    let barrier = Arc::new(Barrier::new(2));
    let other_barrier = barrier.clone();
    let first = thread::spawn(move || {
        other_barrier.wait();
        drop(db);
    });
    let second = thread::spawn(move || {
        barrier.wait();
        drop(second);
    });
    first.join().unwrap();
    second.join().unwrap();
    let reopened = Database::open(dir.path())?;
    assert_eq!(
        reopened
            .get_region("value")
            .unwrap()
            .create_reader()
            .read_all(),
        b"persisted"
    );
    Ok(())
}

#[test]
fn remapping_wait_does_not_lock_out_region_lookups_or_flushes() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    for index in 0..256 {
        db.create_region_if_needed(&index.to_string())?
            .write(b"value")?;
    }
    let region = db.get_region("0").unwrap();
    let reader = region.create_reader();
    let (started, ready) = mpsc::channel();
    let (done, finished) = mpsc::channel();
    let growing = db.clone();
    let grower = thread::spawn(move || {
        started.send(()).unwrap();
        let _ = growing.create_region_if_needed("requires_remap").unwrap();
        done.send(()).unwrap();
    });
    ready.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(finished.recv_timeout(Duration::from_millis(30)).is_err());
    // Run the lookup/flush on a separate thread so timeout cleanup can release
    // the reader even if a future change reintroduces the lock cycle.
    let checking = db.clone();
    let (checked, result) = mpsc::channel();
    let checker = thread::spawn(move || {
        let region = checking.get_region("0").unwrap();
        assert_eq!(region.create_reader().read_all(), b"value");
        checked.send(checking.flush()).unwrap();
    });
    let checked = result.recv_timeout(Duration::from_secs(5));
    drop(reader);
    checker.join().unwrap();
    grower.join().unwrap();
    checked.expect("remapping must release allocation and mutation locks")?;
    assert_eq!(db.regions().len(), 257);
    Ok(())
}

#[test]
fn concurrent_flushes_and_appends_survive_reopen() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let region = db.create_region_if_needed("values")?;
    let writing = region.clone();
    let writer = thread::spawn(move || {
        for value in 0..2000u64 {
            writing.write(&value.to_le_bytes()).unwrap();
        }
    });
    for _ in 0..20 {
        db.flush()?;
        thread::yield_now();
    }
    writer.join().unwrap();
    db.flush()?;
    drop(region);
    drop(db);
    let db = Database::open(dir.path())?;
    let reader = db.get_region("values").unwrap().create_reader();
    assert_eq!(reader.len(), 2000 * 8);
    for (index, bytes) in reader.read_all().as_chunks::<8>().0.iter().enumerate() {
        assert_eq!(u64::from_le_bytes(*bytes), index as u64);
    }
    Ok(())
}

#[test]
fn test_concurrent_reads() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let db = Arc::new(db);

    let region = db.create_region_if_needed("shared")?;
    let data = b"Shared data for concurrent reads";
    region.write(data)?;

    // Multiple threads reading simultaneously
    let handles: Vec<_> = (0..20)
        .map(|_| {
            let db = Arc::clone(&db);
            thread::spawn(move || {
                let regions = db.regions();
                let region = regions.get("shared").unwrap();
                let reader = region.create_reader();
                assert_eq!(reader.read_all(), b"Shared data for concurrent reads");
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}
