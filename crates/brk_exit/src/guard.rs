use parking_lot::{ArcRwLockReadGuard, RawRwLock};

use crate::{Exit, INSTALLED};

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

    /// Whether a shutdown signal waits for the caller: an [`ExitGuard`] is alive in the process,
    /// or the shutdown itself runs. Always true in a process that installed no signal handlers
    /// (tests, offline tools): it has no soft quit to wait. The storage crates `debug_assert!`
    /// it in their write and commit primitives, so a debug build panics on a write a signal
    /// could cut.
    ///
    /// Process-wide, because guards move between threads and guarded work runs on threads that
    /// never take one. So a missing guard goes unnoticed while any other thread holds one: this
    /// catches the unguarded write that runs alone, and proves nothing about the rest.
    pub fn is_locked() -> bool {
        INSTALLED.get().is_none_or(|lock| lock.is_locked())
    }
}
