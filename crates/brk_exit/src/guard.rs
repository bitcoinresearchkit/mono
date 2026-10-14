use parking_lot::{ArcRwLockReadGuard, RawRwLock};

use crate::Exit;

/// Owned read guard for [`crate::Exit`]. Can be moved across threads.
pub struct ExitGuard {
    _guard: ArcRwLockReadGuard<RawRwLock, ()>,
}

impl Exit {
    /// Acquires a read lock to protect a critical section from shutdown.
    /// The shutdown thread will wait for all read locks to be released before exiting.
    /// Returns an owned guard that is Send + 'static (can be moved to background threads).
    pub fn lock(&self) -> ExitGuard {
        // Recursive: an update can acquire nested guards after shutdown starts waiting.
        // Those readers must finish before the shutdown writer can proceed.
        ExitGuard {
            _guard: self.lock.read_arc_recursive(),
        }
    }
}
