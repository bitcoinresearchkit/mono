use std::{
    fs,
    sync::{
        Barrier,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

use lsm_tree::Error as TreeError;
use tempfile::tempdir;

use crate::{Database, Error, KeyspaceCreateOptions, Result};

#[test]
fn lock_outlives_every_handle_and_wins_over_an_invalid_marker() -> Result<()> {
    let folder = tempdir()?;

    let database = Database::builder(&folder).open()?;
    let keyspace = database.keyspace("default", KeyspaceCreateOptions::default)?;
    drop(database);
    // Open keyspaces keep the lock.
    assert!(matches!(
        Database::builder(&folder).open(),
        Err(Error::Locked)
    ));
    // So do read views and returned iterators, which pin tables.
    let view = keyspace.read_view();
    let iter = keyspace.range_as::<[u8; 1], [u8; 1], &[u8], _>(..);
    drop(keyspace);
    assert!(matches!(
        Database::builder(&folder).open(),
        Err(Error::Locked)
    ));
    let prefix = view.prefix_as::<[u8; 1], [u8; 1]>(&[]);
    drop((view, iter));
    assert!(matches!(
        Database::builder(&folder).open(),
        Err(Error::Locked)
    ));
    drop(prefix);

    // The lock file stays behind and is reacquired.
    let database = Database::builder(&folder).open()?;
    fs::write(folder.path().join("version"), b"FJL\x08")?;
    assert!(matches!(
        Database::builder(&folder).open(),
        Err(Error::Locked)
    ));
    drop(database);
    assert!(matches!(
        Database::builder(&folder).open(),
        Err(Error::InvalidVersion)
    ));
    Ok(())
}

#[test]
fn concurrent_same_name_opens_once() -> Result<()> {
    const THREADS: usize = 8;

    let folder = tempdir()?;
    let database = Database::builder(&folder).open()?;
    let barrier = Barrier::new(THREADS);
    let opens = AtomicUsize::new(0);

    thread::scope(|scope| {
        let handles = (0..THREADS)
            .map(|_| {
                scope.spawn(|| {
                    barrier.wait();
                    database.keyspace("shared", || {
                        opens.fetch_add(1, Ordering::Relaxed);
                        thread::sleep(Duration::from_millis(10));
                        KeyspaceCreateOptions::default()
                    })
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .try_for_each(|handle| handle.join().expect("worker thread panicked").map(drop))
    })?;

    assert_eq!(opens.load(Ordering::Relaxed), 1);
    Ok(())
}

#[test]
fn concurrent_ingestions_and_tombstones_survive_recovery() -> Result<()> {
    fn records() -> KeyspaceCreateOptions {
        KeyspaceCreateOptions::default().compaction_records::<[u8; 1], [u8; 2]>()
    }
    let directory = tempdir()?;

    {
        let database = Database::builder(&directory).open()?;
        let first = database.keyspace("first", records)?;
        let second = database.keyspace("second", records)?;

        thread::scope(|scope| {
            let handles = [(&first, *b"a"), (&second, *b"b")].map(|(keyspace, key)| {
                scope.spawn(move || {
                    let mut ingestion = keyspace.start_ingestion_as::<[u8; 1], [u8; 2]>()?;
                    ingestion.write(key, [1; 2])?;
                    ingestion.write(*b"c", [2; 2])?;
                    ingestion.finish()
                })
            });
            handles
                .into_iter()
                .try_for_each(|handle| handle.join().expect("worker thread panicked"))
        })?;

        let mut ingestion = first.start_ingestion_as::<[u8; 1], [u8; 2]>()?;
        ingestion.write_weak_tombstone(*b"a")?;
        ingestion.write(*b"d", [3; 2])?;
        ingestion.finish()?;
    }

    let database = Database::builder(&directory).open()?;
    let first = database.keyspace("first", records)?;
    let second = database.keyspace("second", records)?;
    assert_eq!(first.get_as::<[u8; 2]>(b"a")?, None);
    assert_eq!(first.get_as::<[u8; 2]>(b"c")?, Some([2; 2]));
    assert_eq!(first.get_as::<[u8; 2]>(b"d")?, Some([3; 2]));
    assert_eq!(second.get_as::<[u8; 2]>(b"b")?, Some([1; 2]));
    assert_eq!(first.range_as::<[u8; 1], [u8; 2], &[u8], _>(..).count(), 2);
    Ok(())
}

#[test]
fn compaction_uses_configured_record_widths() -> Result<()> {
    let directory = tempdir()?;
    let database = Database::builder(&directory).open()?;
    let keyspace = database.keyspace("records", || {
        KeyspaceCreateOptions::default().compaction_records::<[u8; 8], [u8; 4]>()
    })?;

    // Prepare runs directly so the background worker cannot race this check.
    for generation in 0..4_u8 {
        let mut ingest = keyspace.tree().ingestion()?;
        ingest.write(1_u64.to_be_bytes(), [generation; 3])?;
        ingest.finish()?;
    }
    let version = keyspace.tree().current_version_id();
    assert!(matches!(
        keyspace.compact(),
        Err(Error::Storage(TreeError::InvalidRecordLength {
            expected: 4,
            actual: 3,
        }))
    ));
    assert_eq!(keyspace.tree().current_version_id(), version);
    assert_eq!(
        keyspace.get_as::<[u8; 3]>(&1_u64.to_be_bytes())?,
        Some([3; 3])
    );
    Ok(())
}
