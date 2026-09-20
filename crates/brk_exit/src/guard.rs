use std::sync::Arc;

use parking_lot::{
    RwLock,
    lock_api::{RawRwLock, RawRwLockRecursive},
};

use crate::Exit;

/// Owned read guard for [`crate::Exit`]. Can be moved across threads.
///
/// parking_lot's `RawRwLock` supports cross-thread unlock,
/// so sending this guard to another thread is safe.
pub struct ExitGuard(Arc<RwLock<()>>);

impl ExitGuard {
    fn new(lock: &Arc<RwLock<()>>) -> Self {
        let arc = Arc::clone(lock);
        // SAFETY: we release the lock in Drop.
        // An update can acquire nested guards after shutdown starts waiting.
        // Those readers must finish before the shutdown writer can proceed.
        unsafe { arc.raw().lock_shared_recursive() };
        Self(arc)
    }
}

impl Exit {
    /// Acquires a read lock to protect a critical section from shutdown.
    /// The shutdown thread will wait for all read locks to be released before exiting.
    /// Returns an owned guard that is Send + 'static (can be moved to background threads).
    pub fn lock(&self) -> ExitGuard {
        ExitGuard::new(&self.lock)
    }
}

impl Drop for ExitGuard {
    fn drop(&mut self) {
        // SAFETY: we acquired the shared lock in `new`, so we must release it.
        unsafe { self.0.raw().unlock_shared() };
    }
}

// SAFETY: parking_lot's RawRwLock supports unlock from a different thread than lock.
unsafe impl Send for ExitGuard {}
unsafe impl Sync for ExitGuard {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, thread, time::Duration};

    #[test]
    fn nested_guard_can_finish_while_shutdown_waits() {
        let exit = Exit::new();
        let outer = exit.lock();
        let (started, waiting) = mpsc::channel();
        let shutdown = exit.clone();
        let writer = thread::spawn(move || {
            started.send(()).unwrap();
            let _shutdown = shutdown.lock.write();
        });
        waiting.recv().unwrap();
        thread::sleep(Duration::from_millis(30));
        let (done, finished) = mpsc::channel();
        let nested = thread::spawn(move || {
            let _nested = exit.lock();
            done.send(()).unwrap();
        });
        let completed = finished.recv_timeout(Duration::from_secs(2));
        // Release and join even on failure, so the test itself cannot deadlock.
        drop(outer);
        nested.join().unwrap();
        writer.join().unwrap();
        completed.expect("nested guard waited for shutdown while holding its outer guard");
    }
}
