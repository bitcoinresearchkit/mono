use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

/// Budget gate for [`super::CachedVec`] materialization.
///
/// When the budget is exhausted, reads fall through to the inner vec without caching.
pub trait CachedVecBudget: Send + Sync {
    /// Returns whether this read may populate an empty cache.
    fn admit(&self, cache_worthy: bool) -> bool {
        cache_worthy
    }

    /// Records an access and returns a monotonically increasing recency stamp.
    fn record_access(&self) -> u64 {
        0
    }

    /// Attempts to reserve `bytes` for an admitted snapshot.
    fn try_reserve(&self, bytes: usize) -> bool;

    /// Releases bytes previously reserved by [`Self::try_reserve`].
    fn release(&self, bytes: usize);
}

impl CachedVecBudget for AtomicUsize {
    #[inline]
    fn try_reserve(&self, bytes: usize) -> bool {
        self.fetch_update(Relaxed, Relaxed, |n| n.checked_sub(bytes))
            .is_ok()
    }

    #[inline]
    fn release(&self, bytes: usize) {
        self.fetch_add(bytes, Relaxed);
    }
}

/// Budget that always allows materialization (used by [`super::CachedVec::wrap`]).
pub struct NoBudget;

impl CachedVecBudget for NoBudget {
    #[inline]
    fn admit(&self, _: bool) -> bool {
        true
    }

    #[inline]
    fn try_reserve(&self, _: usize) -> bool {
        true
    }

    #[inline]
    fn release(&self, _: usize) {}
}
