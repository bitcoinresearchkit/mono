use std::{
    ops::RangeBounds,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use inner::Inner;
use lsm_tree::{RecordBytes, Slice};
use options::CreateOptions;

use crate::{
    Result,
    db_config::Config,
    file::KEYSPACES_FOLDER,
    locked_file::LockedFileGuard,
    worker_pool::{WorkerMessage, WorkerPool},
};

mod ingestion;
mod inner;
pub mod options;

pub use ingestion::Ingestion;

/// A keyspace's table layout pinned for reads: later ingestions and compactions stay invisible
/// to it, and the tables it reads stay on disk while it lives.
#[derive(Clone)]
pub struct ReadView(lsm_tree::ReadView);

impl ReadView {
    /// Iterates over the pinned records matching `prefix`, using the specified owned record bytes.
    #[must_use]
    pub fn prefix_as<K: RecordBytes, V: RecordBytes>(
        &self,
        prefix: &[u8],
    ) -> impl DoubleEndedIterator<Item = Result<(K, V)>> + Send + 'static + use<K, V> {
        self.0
            .prefix_as(prefix)
            .map(|item| item.map_err(Into::into))
    }
}

/// A named, table-only LSM keyspace.
#[derive(Clone)]
pub struct Keyspace {
    inner: Arc<Inner>,
}

impl Keyspace {
    /// Opens or creates a named keyspace.
    #[doc(hidden)]
    pub(crate) fn open(
        name: &str,
        options: CreateOptions,
        database: &Config,
        worker_pool: &WorkerPool,
        lock: LockedFileGuard,
    ) -> Result<Self> {
        let path = database.path.join(KEYSPACES_FOLDER).join(name);
        let compaction = options.compaction;
        let tree = options.tree_config(&path, database).open()?;

        let keyspace = Self {
            inner: Arc::new(Inner {
                name: name.to_owned(),
                tree,
                compaction,
                worker: worker_pool.sender(),
                compaction_queued: AtomicBool::new(false),
                _lock: lock,
            }),
        };

        if keyspace.inner.tree.l0_run_count() > 0 {
            keyspace.request_compaction();
        }

        Ok(keyspace)
    }

    #[cfg(test)]
    pub(crate) fn tree(&self) -> &lsm_tree::Tree {
        &self.inner.tree
    }

    /// Returns the keyspace name.
    #[must_use]
    pub(crate) fn name(&self) -> &str {
        &self.inner.name
    }

    /// Starts ingestion with the specified record representation.
    ///
    /// # Errors
    /// Returns an error if an output table cannot be created.
    pub fn start_ingestion_as<K: RecordBytes, V: RecordBytes>(
        &self,
    ) -> Result<Ingestion<'_, K, V>> {
        Ingestion::new(self)
    }

    /// Reads a value into the specified owned bytes.
    ///
    /// # Errors
    /// Returns an error if the table cannot be read or the value width does not match.
    pub fn get_as<V: RecordBytes>(&self, key: &[u8]) -> Result<Option<V>> {
        self.inner.tree.get_as(key).map_err(Into::into)
    }

    /// Iterates over a range using the specified owned record bytes.
    #[must_use]
    pub fn range_as<K: RecordBytes, V: RecordBytes, B: AsRef<[u8]>, R: RangeBounds<B>>(
        &self,
        range: R,
    ) -> impl DoubleEndedIterator<Item = Result<(K, V)>> + Send + 'static + use<K, V, B, R> {
        self.inner
            .tree
            .range_as(range)
            .map(|item| item.map_err(Into::into))
    }

    /// Iterates over matching keys using the specified owned record bytes.
    #[must_use]
    pub fn prefix_as<K: RecordBytes, V: RecordBytes>(
        &self,
        prefix: &[u8],
    ) -> impl DoubleEndedIterator<Item = Result<(K, V)>> + Send + 'static + use<K, V> {
        self.inner
            .tree
            .prefix_as(prefix)
            .map(|item| item.map_err(Into::into))
    }

    /// Pins the keyspace's current table layout for later reads.
    #[must_use]
    pub fn read_view(&self) -> ReadView {
        ReadView(self.inner.tree.read_view())
    }

    /// Iterates over all latest key-value pairs.
    #[must_use]
    fn iter(&self) -> impl DoubleEndedIterator<Item = Result<(Slice, Slice)>> + Send + 'static {
        self.inner.tree.iter().map(|item| item.map_err(Into::into))
    }

    /// Returns whether the keyspace has no visible values.
    ///
    /// # Errors
    ///
    /// Returns an error if the first table entry cannot be read.
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.iter().next().transpose()?.is_none())
    }

    /// Runs leveled compaction until no eligible work remains.
    #[doc(hidden)]
    pub(crate) fn compact(&self) -> Result<()> {
        (self.inner.compaction)(&self.inner.tree)?;
        Ok(())
    }

    /// Runs a queued compaction. Ingestions finishing meanwhile queue a follow-up.
    pub(crate) fn compact_queued(&self) -> Result<()> {
        // A swap (not a store) also acquires the version the requester published.
        self.inner.compaction_queued.swap(false, Ordering::AcqRel);
        self.compact()
    }

    /// Queues this keyspace for background compaction unless a request is already pending.
    fn request_compaction(&self) {
        if !self.inner.compaction_queued.swap(true, Ordering::AcqRel) {
            // Fails only once the database dropped its workers.
            let _ = self.inner.worker.send(WorkerMessage::Compact(self.clone()));
        }
    }
}
