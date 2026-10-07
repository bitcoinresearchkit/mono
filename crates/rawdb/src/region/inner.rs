use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::RwLock;

use crate::{Database, database::weak::WeakDatabase};

use super::RegionMetadata;

#[derive(Debug)]
pub(crate) struct RegionInner {
    pub(crate) db: WeakDatabase,
    pub(crate) index: usize,
    accessed: AtomicBool,
    pub(crate) meta: RwLock<RegionMetadata>,
    /// Readers hold this for the lifetime of their bytes; mutations hold it exclusively.
    pub(crate) access: RwLock<()>,
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
            tail_needs_punch: AtomicBool::new(false),
        }
    }

    /// # Safety
    /// Hold region access or the exclusive database mutation barrier.
    /// Every metadata mutation takes both region access and the shared barrier.
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
