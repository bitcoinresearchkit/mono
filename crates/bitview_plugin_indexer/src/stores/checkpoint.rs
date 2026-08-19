use brk_error::Result;

use std::{
    fs::{self, File},
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use brk_store::PendingIngest;
use brk_types::Height;
use fjall::Database;
use rayon::prelude::*;

/// Sole durability marker for the shared stores database.
///
/// The file contains the next block height to index. A missing or malformed
/// file means a commit was interrupted and the stores must not be resumed.
#[derive(Debug, Clone)]
pub struct StoresCheckpoint {
    path: PathBuf,
}

impl StoresCheckpoint {
    pub fn new(stores_path: &Path) -> Self {
        Self {
            path: stores_path.join("height"),
        }
    }

    pub fn next_height(&self) -> Result<Option<Height>> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err.into()),
        };
        let Ok(bytes) = <[u8; size_of::<u32>()]>::try_from(bytes) else {
            return Ok(None);
        };
        Ok(Some(Height::new(u32::from_le_bytes(bytes))))
    }

    pub fn initialize_empty(&self) -> Result<()> {
        let pending_path = self.invalidate()?;
        PersistedStoresCheckpoint(PendingStoresCheckpoint {
            next_height: Height::ZERO,
            path: self.path.clone(),
            pending_path,
        })
        .publish()
    }

    pub fn begin(&self, completed_height: Height) -> Result<PendingStoresCheckpoint> {
        let pending_path = self.invalidate()?;

        Ok(PendingStoresCheckpoint {
            next_height: completed_height.incremented(),
            path: self.path.clone(),
            pending_path,
        })
    }

    fn invalidate(&self) -> Result<PathBuf> {
        let pending_path = self.path.with_extension("pending");
        let removed_checkpoint = remove_if_exists(&self.path)?;
        let removed_pending = remove_if_exists(&pending_path)?;

        if removed_checkpoint || removed_pending {
            sync_parent(&self.path)?;
        }

        Ok(pending_path)
    }
}

#[must_use = "dropping a pending checkpoint leaves the stores checkpoint invalid"]
pub struct PendingStoresCheckpoint {
    next_height: Height,
    path: PathBuf,
    pending_path: PathBuf,
}

impl PendingStoresCheckpoint {
    pub fn persist(
        self,
        _db: &Database,
        ingest: impl FnOnce() -> Result<()>,
    ) -> Result<PersistedStoresCheckpoint> {
        ingest()?;
        Ok(PersistedStoresCheckpoint(self))
    }
}

#[must_use = "publish this checkpoint after every related database is durable"]
pub struct PersistedStoresCheckpoint(PendingStoresCheckpoint);

impl PersistedStoresCheckpoint {
    pub fn publish(self) -> Result<()> {
        let pending = self.0;
        pending.next_height.write(&pending.pending_path)?;
        File::open(&pending.pending_path)?.sync_all()?;
        fs::rename(&pending.pending_path, &pending.path)?;
        sync_parent(&pending.path)?;
        Ok(())
    }
}

#[must_use = "persist this deferred commit before publishing its checkpoint"]
pub struct DeferredStoresCommit {
    checkpoint: PendingStoresCheckpoint,
    db: Database,
    ingests: Vec<PendingIngest>,
}

impl DeferredStoresCommit {
    pub fn new(
        db: Database,
        ingests: Vec<PendingIngest>,
        checkpoint: PendingStoresCheckpoint,
    ) -> Self {
        Self {
            checkpoint,
            db,
            ingests,
        }
    }

    pub fn persist(self) -> Result<PersistedStoresCheckpoint> {
        self.checkpoint.persist(&self.db, || {
            self.ingests
                .into_par_iter()
                .try_for_each(PendingIngest::run)
        })
    }
}

fn remove_if_exists(path: &Path) -> io::Result<bool> {
    match fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err),
    }
}

fn sync_parent(path: &Path) -> io::Result<()> {
    File::open(path.parent().expect("checkpoint has a parent"))?.sync_all()
}

#[cfg(test)]
mod tests {
    use brk_error::Error;
    use brk_store::{Kind, Mode, Store};
    use brk_types::{AddrIndexTxIndex, TxIndex, TypeIndex, Unit, Version};

    use super::*;

    fn key(address: u32, transaction: u32) -> AddrIndexTxIndex {
        AddrIndexTxIndex::from((TypeIndex::new(address), TxIndex::new(transaction)))
    }

    fn open_store(db: &Database, path: &Path, name: &str) -> Result<Store<AddrIndexTxIndex, Unit>> {
        Store::import(db, path, name, Version::ZERO, Mode::Any, Kind::Vec)
    }

    #[test]
    fn dropped_commit_leaves_checkpoint_invalid() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let checkpoint = StoresCheckpoint::new(dir.path());
        let db = brk_store::open_database(dir.path())?;

        checkpoint
            .begin(Height::new(41))?
            .persist(&db, || Ok(()))?
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
                .persist(&db, || Ok(()))?
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
}
