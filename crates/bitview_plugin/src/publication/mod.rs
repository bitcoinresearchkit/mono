use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use parking_lot::{ArcRwLockWriteGuard, Mutex, RawRwLock, RwLock};

mod read_guard;

pub use read_guard::PublicationReadGuard;

/// Shared publication barrier for one complete Bitview pipeline.
///
/// Clones refer to the same gate. An update stays closed until
/// [`finish_update`](Self::finish_update) is called explicitly, so an error
/// cannot expose partially updated mutable state.
#[derive(Clone, Default)]
pub struct Publication(Arc<Inner>);

#[derive(Default)]
struct Inner {
    gate: Arc<RwLock<()>>,
    writer: Mutex<Option<ArcRwLockWriteGuard<RawRwLock, ()>>>,
    revision: AtomicU64,
}

impl Publication {
    /// Waits for existing readers, then closes the pipeline to mutable reads.
    /// Calling this while the same update is already running is a no-op.
    pub fn begin_update(&self) {
        let mut writer = self.0.writer.lock();
        if writer.is_some() {
            return;
        }

        *writer = Some(self.0.gate.write_arc());
    }

    /// Publishes the completed update and admits new readers.
    ///
    /// # Panics
    ///
    /// Panics when no update is running.
    pub fn finish_update(&self) {
        let writer = self
            .0
            .writer
            .lock()
            .take()
            .expect("pipeline update is not running");
        self.0.revision.fetch_add(1, Ordering::Release);
        drop(writer);
    }

    /// Process-local revision of completed publications, shared by gate clones.
    /// Read while holding this barrier when pairing the revision with pipeline data.
    /// It advances even if a publication leaves the chain tip unchanged.
    pub fn revision(&self) -> u64 {
        self.0.revision.load(Ordering::Acquire)
    }

    /// Whether both handles are the same gate (clones of one publication).
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
