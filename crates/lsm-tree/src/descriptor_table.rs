// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{fs::File, io::Result, path::Path, sync::Arc};

use quick_cache::{UnitWeighter, sync::Cache as QuickCache};
use rustc_hash::FxBuildHasher;

use crate::GlobalTableId;

/// Caches file descriptors to tables
pub struct DescriptorTable {
    inner: QuickCache<GlobalTableId, Arc<File>, UnitWeighter, FxBuildHasher>,
}

impl DescriptorTable {
    /// Creates a descriptor cache able to retain up to `capacity` open files.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        use quick_cache::sync::DefaultLifecycle;

        let quick_cache = QuickCache::with(
            1_000,
            capacity as u64,
            UnitWeighter,
            FxBuildHasher,
            DefaultLifecycle::default(),
        );

        Self { inner: quick_cache }
    }

    /// Returns the cached descriptor, opening it exactly once on a concurrent miss.
    ///
    /// # Errors
    ///
    /// Returns an error if the table file cannot be opened.
    pub(crate) fn access_or_open(&self, id: GlobalTableId, path: &Path) -> Result<Arc<File>> {
        self.inner
            .get_or_insert_with(&id, || File::open(path).map(Arc::new))
    }

    /// Removes a table's descriptor from the cache.
    pub(crate) fn remove_for_table(&self, id: GlobalTableId) {
        self.inner.remove(&id);
    }
}
