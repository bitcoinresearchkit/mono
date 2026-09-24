use std::{ops::Deref, sync::Arc};

use super::inner::DatabaseInner;

/// Foreground handles share one owner that joins workers before releasing storage.
/// Workers share its background owner, which keeps storage alive without joining.
// Keep storage off the reference-counter cache line during concurrent upgrades.
#[repr(align(128))]
pub(crate) struct DatabaseOwner {
    pub(super) storage: Arc<DatabaseInner>,
    pub(super) background: Option<Arc<Self>>,
}

impl Deref for DatabaseOwner {
    type Target = DatabaseInner;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl Drop for DatabaseOwner {
    fn drop(&mut self) {
        if self.background.is_some() {
            let _ = self.tasks.join();
        }
    }
}
