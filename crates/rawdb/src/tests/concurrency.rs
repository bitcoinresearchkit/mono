use std::{
    collections::BTreeMap,
    sync::{
        Arc, Barrier,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
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
fn scoped_read_releases_access_before_joining_deferred_writer() -> Result<()> {
    for created_on_worker in [false, true] {
        let dir = TempDir::new()?;
        let db = Database::open(dir.path())?;
        if created_on_worker {
            db.run_bg(|db| db.create_region_if_needed("value").map(|_| ()));
            db.sync_bg_tasks()?;
        }
        let region = db.create_region_if_needed("value")?;
        region.write(b"before")?;
        let writer = region.clone();
        db.run_bg(move |db| {
            db.bg_sleep(Duration::from_secs(60));
            writer.write_at(b"after!", 0)
        });

        let (done, finished) = mpsc::channel();
        let reader = thread::spawn(move || {
            region.with_read_bytes(move |bytes| {
                assert_eq!(bytes, b"before");
                drop(db);
            });
            done.send(()).unwrap();
        });
        finished.recv_timeout(Duration::from_secs(5)).unwrap();
        reader.join().unwrap();

        let db = Database::open(dir.path())?;
        assert_eq!(
            db.get_region("value").unwrap().create_reader().read_all(),
            b"after!"
        );
    }
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
fn batch_callback_can_read_another_region_while_remapping_waits() -> Result<()> {
    for indexed in [false, true] {
        let dir = TempDir::new()?;
        let db = Database::open(dir.path())?;
        let output = db.create_region_if_needed("output")?;
        output.write(&[0; 8])?;
        let input = db.create_region_if_needed("input")?;
        input.write(b"source")?;
        let length = db.file_len();
        let (entered, ready) = mpsc::channel();
        let (continue_reading, proceed) = mpsc::channel();
        let (completed, done) = mpsc::channel();
        let writer = thread::spawn(move || {
            let write = |value: &u64, bytes: &mut [u8]| {
                entered.send(()).unwrap();
                proceed.recv().unwrap();
                assert_eq!(input.create_reader().read_all(), b"source");
                bytes.copy_from_slice(&value.to_le_bytes());
            };
            if indexed {
                output.write_indexed(BTreeMap::from([(0, 7u64)]), 8, 0, write);
            } else {
                output.batch_write_ordered([(0, 7u64)].into_iter(), 8, write);
            }
            completed.send(()).unwrap();
        });
        ready.recv_timeout(Duration::from_secs(5)).unwrap();
        let growing = db.clone();
        let (started, beginning) = mpsc::channel();
        let grower = thread::spawn(move || {
            started.send(()).unwrap();
            growing.set_min_len(length * 2).unwrap();
        });
        beginning.recv_timeout(Duration::from_secs(5)).unwrap();
        thread::sleep(Duration::from_millis(30));
        continue_reading.send(()).unwrap();
        done.recv_timeout(Duration::from_secs(5)).unwrap();
        writer.join().unwrap();
        grower.join().unwrap();
        assert_eq!(
            db.get_region("output").unwrap().create_reader().read_all(),
            &7u64.to_le_bytes()
        );
    }
    Ok(())
}

#[test]
fn remapping_completes_while_batches_keep_reading_and_writing() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let input = db.create_region_if_needed("input")?;
    input.write(b"source")?;
    let stop = Arc::new(AtomicBool::new(false));
    let ready = Arc::new(Barrier::new(5));
    let mut writers = Vec::new();
    for index in 0..4 {
        let output = db.create_region_if_needed(&format!("output_{index}"))?;
        output.write(&[0; 8])?;
        let input = input.clone();
        let stop = stop.clone();
        let ready = ready.clone();
        writers.push(thread::spawn(move || {
            ready.wait();
            while !stop.load(Ordering::Relaxed) {
                output.batch_write_ordered([(0, 7u64)].into_iter(), 8, |value, bytes| {
                    assert_eq!(input.create_reader().read_all(), b"source");
                    bytes.copy_from_slice(&value.to_le_bytes());
                });
            }
        }));
    }
    ready.wait();
    let length = db.file_len();
    let growing = db.clone();
    let (completed, done) = mpsc::channel();
    let grower = thread::spawn(move || {
        completed.send(growing.set_min_len(length * 16)).unwrap();
    });
    let result = done.recv_timeout(Duration::from_secs(5));
    stop.store(true, Ordering::Relaxed);
    for writer in writers {
        writer.join().unwrap();
    }
    grower.join().unwrap();
    result.expect("continuous batches must not starve remapping")?;
    assert!(db.file_len() >= length * 16);
    Ok(())
}

#[test]
fn background_handle_keeps_regions_usable_after_foreground_shutdown() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let region = db.create_region_if_needed("value")?;
    let (send, receive) = mpsc::channel();
    db.run_bg(move |db| {
        send.send(db.clone()).unwrap();
        Ok(())
    });
    let background = receive.recv_timeout(Duration::from_secs(5)).unwrap();
    drop(db);
    region.write(b"after foreground shutdown")?;
    background.run_bg(|db| db.flush().map(|_| ()));
    background.sync_bg_tasks()?;
    assert_eq!(
        region.create_reader().read_all(),
        b"after foreground shutdown"
    );
    drop(region);
    drop(background);
    let reopened = Database::open(dir.path())?;
    assert_eq!(
        reopened
            .get_region("value")
            .unwrap()
            .create_reader()
            .read_all(),
        b"after foreground shutdown"
    );
    Ok(())
}

#[test]
fn concurrent_relocations_and_flushes_preserve_every_region() -> Result<()> {
    let dir = TempDir::new()?;
    {
        let db = Database::open(dir.path())?;
        db.set_min_len(16 * 1024 * 1024)?;
        let mut regions = Vec::new();
        for id in 0..4u8 {
            let region = db.create_region_if_needed(&id.to_string())?;
            region.write(&vec![id; 1024 * 1024])?;
            regions.push(region);
        }
        // Block in-place growth, so each writer must copy to a new allocation.
        drop(db.create_region_if_needed("blocker")?);
        db.flush()?;
        let barrier = Barrier::new(regions.len() + 1);
        thread::scope(|scope| -> Result<()> {
            let handles: Vec<_> = regions
                .iter()
                .map(|region| {
                    let barrier = &barrier;
                    scope.spawn(move || {
                        barrier.wait();
                        region.write(&[9])
                    })
                })
                .collect();
            barrier.wait();
            db.flush()?;
            for handle in handles {
                handle.join().unwrap()?;
            }
            Ok(())
        })?;
        db.flush()?;
    }
    let db = Database::open(dir.path())?;
    for id in 0..4u8 {
        let reader = db.get_region(&id.to_string()).unwrap().create_reader();
        assert_eq!(reader.len(), 1024 * 1024 + 1);
        assert!(reader.read_all()[..1024 * 1024].iter().all(|&b| b == id));
        assert_eq!(reader.read_all().last(), Some(&9));
    }
    Ok(())
}
