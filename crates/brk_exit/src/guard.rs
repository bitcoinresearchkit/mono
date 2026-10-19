use parking_lot::{ArcRwLockReadGuard, RawRwLock};

use crate::Exit;

/// Defers shutdown while held; `Send + 'static`, so it can move to background threads.
pub struct ExitGuard {
    _guard: ArcRwLockReadGuard<RawRwLock, ()>,
}

impl Exit {
    /// Protects a critical section: the shutdown thread waits until every guard is dropped.
    pub fn lock(&self) -> ExitGuard {
        // Recursive: an update can acquire nested guards after shutdown starts waiting.
        // Those readers must finish before the shutdown writer can proceed.
        ExitGuard {
            _guard: self.lock.read_arc_recursive(),
        }
    }
}
