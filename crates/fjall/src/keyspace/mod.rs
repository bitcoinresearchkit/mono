mod ingestion;
mod inner;
pub mod options;

use crate::{
    db_config::Config,
    file::KEYSPACES_FOLDER,
    locked_file::LockedFileGuard,
    worker_pool::{WorkerMessage, WorkerPool},
};
use inner::Inner;
use options::CreateOptions;
use std::{ops::RangeBounds, sync::Arc};

pub use ingestion::Ingestion;

/// A named, table-only LSM keyspace.
#[derive(Clone)]
pub struct Keyspace {
    inner: Arc<Inner>,
}

impl Keyspace {
    /// Opens or creates a named keyspace.
    #[doc(hidden)]
    pub fn open(
        name: &str,
        options: CreateOptions,
        database: &Config,
        worker_pool: &WorkerPool,
        lock: LockedFileGuard,
    ) -> crate::Result<Self> {
        let path = database.path.join(KEYSPACES_FOLDER).join(name);
        let tree = options.tree_config(&path, database).open()?;

        let keyspace = Self {
            inner: Arc::new(Inner {
                name: name.to_owned(),
                tree,
                worker: worker_pool.sender(),
                _lock: lock,
            }),
        };

        if keyspace.inner.tree.l0_run_count() > 0 {
            keyspace.request_compaction();
        }

        Ok(keyspace)
    }

    /// Returns the keyspace name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// Starts a sorted bulk ingestion directly into `SSTables`.
    ///
    /// # Errors
    ///
    /// Returns an error if an output table cannot be created.
    pub fn start_ingestion(&self) -> crate::Result<Ingestion<'_>> {
        Ingestion::new(self)
    }

    /// Reads the latest value for `key` from immutable tables.
    ///
    /// # Errors
    ///
    /// Returns an error if a table cannot be read or decoded.
    pub fn get<K: AsRef<[u8]>>(&self, key: K) -> crate::Result<Option<lsm_tree::Slice>> {
        self.inner.tree.get(key).map_err(Into::into)
    }

    /// Iterates over all latest key-value pairs.
    #[must_use]
    pub fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = crate::Result<(lsm_tree::Slice, lsm_tree::Slice)>>
    + Send
    + 'static {
        self.inner.tree.iter().map(|item| item.map_err(Into::into))
    }

    /// Iterates over the latest key-value pairs in `range`.
    #[must_use]
    pub fn range<K: AsRef<[u8]>, R: RangeBounds<K>>(
        &self,
        range: R,
    ) -> impl DoubleEndedIterator<Item = crate::Result<(lsm_tree::Slice, lsm_tree::Slice)>>
    + Send
    + 'static {
        self.inner
            .tree
            .range(range)
            .map(|item| item.map_err(Into::into))
    }

    /// Iterates over the latest key-value pairs matching `prefix`.
    #[must_use]
    pub fn prefix<K: AsRef<[u8]>>(
        &self,
        prefix: K,
    ) -> impl DoubleEndedIterator<Item = crate::Result<(lsm_tree::Slice, lsm_tree::Slice)>>
    + Send
    + 'static {
        self.inner
            .tree
            .prefix(prefix)
            .map(|item| item.map_err(Into::into))
    }

    /// Returns whether the keyspace has no visible values.
    ///
    /// # Errors
    ///
    /// Returns an error if the first table entry cannot be read.
    pub fn is_empty(&self) -> crate::Result<bool> {
        Ok(self.iter().next().transpose()?.is_none())
    }

    /// Approximates the number of table entries in constant time.
    #[must_use]
    pub fn approximate_len(&self) -> usize {
        self.inner.tree.approximate_len()
    }

    /// Runs leveled compaction until no eligible work remains.
    #[doc(hidden)]
    pub fn compact(&self) -> crate::Result<()> {
        self.inner.tree.compact()?;
        Ok(())
    }

    /// Queues this keyspace for background compaction.
    #[doc(hidden)]
    pub fn request_compaction(&self) {
        let _ = self
            .inner
            .worker
            .try_send(WorkerMessage::Compact(self.clone()));
    }
}
