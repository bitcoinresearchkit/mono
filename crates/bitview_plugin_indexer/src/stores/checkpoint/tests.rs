use brk_error::{Error, Result};
use brk_store::{Kind, PendingIngest, Store};
use brk_types::{AddrIndexTxIndex, Height, TxIndex, TypeIndex, Unit, Version};
use fjall::Database;
use std::{fs, path::Path};

use super::{DeferredStoresCommit, StoresCheckpoint};

fn key(address: u32, transaction: u32) -> AddrIndexTxIndex {
    AddrIndexTxIndex::from((TypeIndex::new(address), TxIndex::new(transaction)))
}

fn open_store(db: &Database, path: &Path, name: &str) -> Result<Store<AddrIndexTxIndex, Unit>> {
    Store::import(db, path, name, Version::ZERO, Kind::Vec)
}

#[test]
fn dropped_commit_leaves_checkpoint_invalid() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let checkpoint = StoresCheckpoint::new(dir.path());

    checkpoint
        .begin(Height::new(41))?
        .persist(|| Ok(()))?
        .publish()?;
    assert_eq!(checkpoint.next_height()?, Some(Height::new(42)));

    let pending = checkpoint.begin(Height::new(42))?;
    assert_eq!(checkpoint.next_height()?, None);
    drop(pending);

    let reopened = StoresCheckpoint::new(dir.path());
    assert_eq!(reopened.next_height()?, None);
    Ok(())
}

#[test]
fn failed_ingest_does_not_publish_checkpoint() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let checkpoint = StoresCheckpoint::new(dir.path());
    let pending = checkpoint.begin(Height::new(42))?;
    let db = brk_store::open_database(dir.path())?;
    let ingests = vec![PendingIngest::new(|| {
        Err(Error::Internal("simulated ingest failure"))
    })];

    assert!(
        DeferredStoresCommit::new(db, ingests, pending)
            .persist()
            .is_err()
    );
    assert_eq!(checkpoint.next_height()?, None);
    Ok(())
}

#[test]
fn persisted_commit_is_not_published_early() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let checkpoint = StoresCheckpoint::new(dir.path());
    let pending = checkpoint.begin(Height::new(42))?;
    let db = brk_store::open_database(dir.path())?;

    let persisted = DeferredStoresCommit::new(db, vec![], pending).persist()?;
    assert_eq!(checkpoint.next_height()?, None);
    persisted.publish()?;

    let reopened = StoresCheckpoint::new(dir.path());
    assert_eq!(reopened.next_height()?, Some(Height::new(43)));
    Ok(())
}

#[test]
fn dropped_deferred_ingest_reopens_without_value_or_checkpoint() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let checkpoint = StoresCheckpoint::new(dir.path());

    {
        let db = brk_store::open_database(dir.path())?;
        let mut store = open_store(&db, dir.path(), "dropped_deferred_ingest")?;

        checkpoint
            .begin(Height::new(41))?
            .persist(|| Ok(()))?
            .publish()?;
        store.insert(key(1, 1), Unit);

        let pending_checkpoint = checkpoint.begin(Height::new(42))?;
        let pending_ingest = store.take_pending_ingest().unwrap();
        drop(pending_ingest);
        drop(pending_checkpoint);
    }

    let db = brk_store::open_database(dir.path())?;
    let store = open_store(&db, dir.path(), "dropped_deferred_ingest")?;

    assert_eq!(checkpoint.next_height()?, None);
    assert!(store.get(&key(1, 1))?.is_none());
    Ok(())
}

#[test]
fn successful_ingest_reopens_with_value_and_checkpoint() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let checkpoint = StoresCheckpoint::new(dir.path());

    {
        let db = brk_store::open_database(dir.path())?;
        let mut store = open_store(&db, dir.path(), "successful_ingest")?;
        store.insert(key(1, 1), Unit);

        let pending_checkpoint = checkpoint.begin(Height::new(42))?;
        let pending_ingest = store.take_pending_ingest().unwrap();
        DeferredStoresCommit::new(db.clone(), vec![pending_ingest], pending_checkpoint)
            .persist()?
            .publish()?;
    }

    let db = brk_store::open_database(dir.path())?;
    let store = open_store(&db, dir.path(), "successful_ingest")?;

    assert_eq!(checkpoint.next_height()?, Some(Height::new(43)));
    assert!(store.get(&key(1, 1))?.is_some());
    Ok(())
}

#[test]
fn empty_database_has_an_explicit_zero_checkpoint() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let checkpoint = StoresCheckpoint::new(dir.path());

    assert_eq!(checkpoint.next_height()?, None);
    checkpoint.initialize_empty()?;
    assert_eq!(checkpoint.next_height()?, Some(Height::ZERO));
    Ok(())
}

#[test]
fn malformed_checkpoint_is_invalid_but_io_errors_propagate() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let checkpoint = StoresCheckpoint::new(dir.path());

    fs::write(&checkpoint.path, [0_u8; 3])?;
    assert_eq!(checkpoint.next_height()?, None);

    fs::remove_file(&checkpoint.path)?;
    fs::create_dir(&checkpoint.path)?;
    assert!(checkpoint.next_height().is_err());
    Ok(())
}
