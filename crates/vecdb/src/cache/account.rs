use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering::Relaxed},
};

use super::{CacheBudget, Charge};

/// Lives as long as any charged allocation, without owning the source cache.
#[derive(Debug)]
pub(super) struct Account {
    pub(super) budget: &'static CacheBudget,
    used: AtomicUsize,
}

impl Account {
    pub(super) fn new(budget: &'static CacheBudget) -> Arc<Self> {
        Arc::new(Self {
            budget,
            used: AtomicUsize::new(0),
        })
    }

    /// Accounted bytes, including buffers still borrowed after eviction.
    pub(super) fn used(&self) -> usize {
        self.used.load(Relaxed)
    }

    pub(super) fn reserve(self: &Arc<Self>, bytes: usize) -> Option<Charge> {
        if !self.budget.reserve(bytes) {
            return None;
        }
        self.used.fetch_add(bytes, Relaxed);
        Some(Charge::new(self.clone(), bytes))
    }

    pub(super) fn release(&self, bytes: usize) {
        let previous = self.used.fetch_sub(bytes, Relaxed);
        debug_assert!(previous >= bytes);
        self.budget.release(bytes);
    }
}
