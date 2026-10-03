use std::{
    collections::VecDeque,
    fmt,
    sync::{
        Weak,
        atomic::{AtomicUsize, Ordering::Relaxed},
    },
};

use parking_lot::Mutex;

pub(super) trait Reclaim: Send + Sync {
    fn try_clear(&self);
    #[cfg(test)]
    fn clear(&self);
}

/// Shared limit for retained buffer and span-directory allocations.
///
/// Charges follow allocation lifetimes, including buffers borrowed by active reads
/// after eviction. Decoder scratch and caller-owned results are not retained bytes.
pub struct CacheBudget {
    limit: usize,
    used: AtomicUsize,
    registry: Mutex<VecDeque<Weak<dyn Reclaim>>>,
}

impl CacheBudget {
    pub(super) const fn new(limit: usize) -> Self {
        Self {
            limit,
            used: AtomicUsize::new(0),
            registry: Mutex::new(VecDeque::new()),
        }
    }

    pub(crate) fn limit(&self) -> usize {
        self.limit
    }

    pub(crate) fn used(&self) -> usize {
        self.used.load(Relaxed)
    }

    pub(super) fn register(&self, cache: Weak<dyn Reclaim>) {
        let mut registry = self.registry.lock();
        // Prune before growing, not on every import: registering many live
        // sources stays amortized linear instead of repeatedly scanning them.
        if registry.len() == registry.capacity() {
            registry.retain(|cache| cache.strong_count() != 0);
        }
        registry.push_back(cache);
    }

    /// Evicts retained data without invalidating the underlying source generation.
    #[cfg(test)]
    pub(crate) fn clear(&self) {
        let caches: Vec<_> = self
            .registry
            .lock()
            .iter()
            .filter_map(Weak::upgrade)
            .collect();
        for cache in caches {
            cache.clear();
        }
    }

    pub(super) fn reserve(&self, bytes: usize) -> bool {
        if bytes > self.limit {
            return false;
        }
        let claim = || {
            self.used
                .fetch_update(Relaxed, Relaxed, |used| {
                    used.checked_add(bytes).filter(|&next| next <= self.limit)
                })
                .is_ok()
        };
        if claim() {
            return true;
        }

        // Two bounded clock passes: a recently used owner gets one second chance.
        // Never wait on a busy cache or call it while holding the registry lock.
        let count = self.registry.lock().len();
        for _ in 0..count.saturating_mul(2) {
            let cache = {
                let mut registry = self.registry.lock();
                let Some(owner) = registry.pop_front() else {
                    break;
                };
                let cache = owner.upgrade();
                if cache.is_some() {
                    registry.push_back(owner);
                }
                cache
            };
            if let Some(cache) = cache {
                cache.try_clear();
            }
            if claim() {
                return true;
            }
        }
        false
    }

    pub(super) fn release(&self, bytes: usize) {
        let previous = self.used.fetch_sub(bytes, Relaxed);
        debug_assert!(previous >= bytes);
    }
}

impl fmt::Debug for CacheBudget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CacheBudget")
            .field("limit", &self.limit)
            .field("used", &self.used())
            .finish()
    }
}
