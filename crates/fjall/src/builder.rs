use std::{path::Path, sync::Arc};

use lsm_tree::{Cache, DescriptorTable};

use crate::{Database, Result, db_config::Config};

/// Builder for BRK's table-only database.
pub struct Builder {
    inner: Config,
}

impl Builder {
    /// Creates a builder rooted at `path`.
    #[must_use]
    pub(crate) fn new(path: &Path) -> Self {
        Self {
            inner: Config::new(path),
        }
    }

    /// Opens the database, creating it if necessary.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be created, recovered, or locked.
    pub fn open(self) -> Result<Database> {
        Database::open(self.inner)
    }

    /// Sets the shared block-cache capacity in bytes.
    #[must_use]
    pub fn cache_size(mut self, bytes: u64) -> Self {
        self.inner.cache = Arc::new(Cache::with_capacity_bytes(bytes));
        self
    }

    /// Sets the number of cached table descriptors.
    #[must_use]
    pub fn max_cached_files(mut self, count: usize) -> Self {
        self.inner.descriptor_table = Arc::new(DescriptorTable::new(count));
        self
    }
}
