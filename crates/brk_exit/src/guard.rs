use std::sync::Arc;

use parking_lot::RwLock;

/// Owned read guard for [`crate::Exit`]. Can be moved across threads.
///
/// parking_lot's `RawRwLock` supports cross-thread unlock,
/// so sending this guard to another thread is safe.
pub struct ExitGuard(Arc<RwLock<()>>);

impl ExitGuard {
    pub(super) fn new(lock: &Arc<RwLock<()>>) -> Self {
        let arc = Arc::clone(lock);
        use parking_lot::lock_api::RawRwLock as _;
        // SAFETY: we release the lock in Drop.
        unsafe { arc.raw().lock_shared() };
        Self(arc)
    }
}

impl Drop for ExitGuard {
    fn drop(&mut self) {
        use parking_lot::lock_api::RawRwLock as _;
        // SAFETY: we acquired the shared lock in `new`, so we must release it.
        unsafe { self.0.raw().unlock_shared() };
    }
}

// SAFETY: parking_lot's RawRwLock supports unlock from a different thread than lock.
unsafe impl Send for ExitGuard {}
unsafe impl Sync for ExitGuard {}
