use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::{Mutex, RwLock};

use crate::{Database, database::weak::WeakDatabase};

use super::RegionMetadata;

#[derive(Debug)]
pub(crate) struct RegionInner {
    pub(crate) db: WeakDatabase,
    pub(crate) index: usize,
    accessed: AtomicBool,
    pub(crate) meta: RwLock<RegionMetadata>,
    /// Readers hold this for the lifetime of their bytes; mutations of bytes or bounds they may
    /// read hold it exclusively, while appends (past every reader's bounds) share it.
    pub(crate) access: RwLock<()>,
    /// Serializes the writes that share `access`.
    pub(crate) writer: Mutex<()>,
    pub(crate) tail_needs_punch: AtomicBool,
}

impl RegionInner {
    pub(super) fn new(db: &Database, index: usize, meta: RegionMetadata) -> Self {
        Self {
            db: WeakDatabase::new(db),
            index,
            accessed: AtomicBool::new(false),
            meta: RwLock::new(meta),
            access: RwLock::new(()),
            writer: Mutex::new(()),
            tail_needs_punch: AtomicBool::new(false),
        }
    }

    /// # Safety
    /// Hold region access or the exclusive database mutation barrier.
    /// Start and reservation change only under exclusive access and the shared barrier; the length
    /// also under shared access, atomically.
    pub(crate) unsafe fn bounds(&self) -> (usize, usize, usize) {
        let meta = unsafe { &*self.meta.data_ptr() };
        (meta.start(), meta.byte_len(), meta.reserved())
    }

    pub(crate) fn mark_accessed(&self) {
        self.accessed.store(true, Ordering::Relaxed);
    }

    pub(crate) fn was_accessed(&self) -> bool {
        self.accessed.load(Ordering::Relaxed)
    }
}
