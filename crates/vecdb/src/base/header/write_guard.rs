use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::{Error, Result};

/// Nested writes share a fence. Only a failed or unwound operation sets it;
/// completing an inner write can never clear an outer operation's failure.
pub(crate) struct WriteGuard {
    failed: Arc<AtomicBool>,
    complete: bool,
}

impl WriteGuard {
    pub(super) fn new(failed: &Arc<AtomicBool>) -> Result<Self> {
        if failed.load(Ordering::Relaxed) {
            return Err(Error::WriteFailed);
        }
        Ok(Self {
            failed: Arc::clone(failed),
            complete: false,
        })
    }

    pub(crate) fn finish<T>(mut self, result: Result<T>) -> Result<T> {
        self.complete = result.is_ok();
        result
    }
}

impl Drop for WriteGuard {
    fn drop(&mut self) {
        if !self.complete {
            self.failed.store(true, Ordering::Relaxed);
        }
    }
}
