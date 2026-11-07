use std::sync::Arc;

use arc_swap::ArcSwapOption;
use parking_lot::Mutex;

use crate::Result;

/// One value derived from a publication, shared by the reads pinning it.
pub struct PublicationCache<T> {
    current: ArcSwapOption<(u64, T)>,
    /// Held by the one read building the value; the others missing it wait, then share it.
    building: Mutex<()>,
}

impl<T> Default for PublicationCache<T> {
    fn default() -> Self {
        Self {
            current: ArcSwapOption::empty(),
            building: Mutex::new(()),
        }
    }
}

impl<T: Clone> PublicationCache<T> {
    /// `build` reads the publication `revision` names (the caller's pin); a read still on an
    /// older publication never replaces a newer entry. One read builds at a time: the others
    /// missing the entry meanwhile wait for it and share it (once a publication), never an
    /// error.
    pub fn get_or_try_init(&self, revision: u64, build: impl FnOnce() -> Result<T>) -> Result<T> {
        if let Some((cached, value)) = self.current.load().as_deref()
            && *cached == revision
        {
            return Ok(value.clone());
        }
        let _building = self.building.lock();
        // Every store happens under `building`: this load sees the latest entry.
        let current = self.current.load_full();
        if let Some((cached, value)) = current.as_deref()
            && *cached == revision
        {
            return Ok(value.clone());
        }
        let value = build()?;
        if current.is_none_or(|current| revision > current.0) {
            self.current
                .store(Some(Arc::new((revision, value.clone()))));
        }
        Ok(value)
    }
}
