use crate::version::Version;
use arc_swap::ArcSwap;
use std::{
    path::Path,
    sync::{Mutex, PoisonError},
};

/// Atomically published latest table version.
pub struct Set {
    latest: ArcSwap<Version>,
    publish_lock: Mutex<()>,
}

impl Set {
    /// Creates a set containing `version`.
    #[must_use]
    pub fn new(version: Version) -> Self {
        Self {
            latest: ArcSwap::from_pointee(version),
            publish_lock: Mutex::default(),
        }
    }

    /// Loads the currently published version.
    #[must_use]
    pub fn load(&self) -> std::sync::Arc<Version> {
        self.latest.load_full()
    }

    /// Borrows the currently published version for a short operation.
    #[must_use]
    pub fn guard(&self) -> arc_swap::Guard<std::sync::Arc<Version>> {
        self.latest.load()
    }

    /// Persists and atomically publishes a version transition.
    pub fn publish(
        &self,
        tree_path: &Path,
        transition: impl FnOnce(&Version) -> crate::Result<Version>,
    ) -> crate::Result<()> {
        let _publish = self
            .publish_lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let current = self.latest.load_full();
        let next = transition(&current)?;

        next.persist(tree_path)?;
        self.latest.store(std::sync::Arc::new(next));

        Ok(())
    }
}
