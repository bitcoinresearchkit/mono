use lsm_tree::{Ingestion as TreeIngestion, RecordBytes, Slice};

use crate::{Keyspace, Result};

/// A strictly sorted stream written directly into `SSTables`.
pub struct Ingestion<'a, K = Slice, V = Slice> {
    keyspace: &'a Keyspace,
    inner: TreeIngestion<'a, K, V>,
}

impl<'a, K: RecordBytes, V: RecordBytes> Ingestion<'a, K, V> {
    /// Starts an ingestion for `keyspace`.
    pub(super) fn new(keyspace: &'a Keyspace) -> Result<Self> {
        let inner = TreeIngestion::new(&keyspace.inner.tree)?;
        Ok(Self { keyspace, inner })
    }

    /// Appends a key-value pair. Keys must be strictly increasing.
    ///
    /// # Errors
    ///
    /// Returns an error if the table writer fails.
    pub fn write<IK: Into<K>, IV: Into<V>>(&mut self, key: IK, value: IV) -> Result<()> {
        self.inner.write(key, value).map_err(Into::into)
    }

    /// Appends a weak tombstone. Keys must be strictly increasing.
    ///
    /// # Errors
    ///
    /// Returns an error if the table writer fails.
    pub fn write_weak_tombstone<IK: Into<K>>(&mut self, key: IK) -> Result<()> {
        self.inner.write_weak_tombstone(key).map_err(Into::into)
    }

    /// Persists and publishes the new immutable tables.
    ///
    /// # Errors
    ///
    /// Returns an error if table or manifest persistence fails.
    pub fn finish(self) -> Result<()> {
        self.inner.finish()?;
        self.keyspace.request_compaction();
        Ok(())
    }
}
