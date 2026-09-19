use std::sync::Arc;

use super::inner::DatabaseInner;

/// Shared by foreground handles; background workers own only the storage.
pub(super) struct DatabaseOwner(pub(super) Arc<DatabaseInner>);

impl Drop for DatabaseOwner {
    fn drop(&mut self) {
        let _ = self.0.tasks.join();
    }
}
