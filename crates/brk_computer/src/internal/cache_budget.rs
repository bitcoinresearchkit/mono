use std::sync::{
    Arc,
    atomic::{AtomicU64, AtomicUsize, Ordering::Relaxed},
};

use parking_lot::Mutex;
use vecdb::{CachedVec, CachedVecBudget, ReadableVec, TypedVec};

const MAX_BYTES: usize = 2 * 1024 * 1024 * 1024;
const MIN_ACCESSES: u64 = 2;

pub struct CacheBudget {
    remaining_bytes: AtomicUsize,
    clock: AtomicU64,
    caches: Mutex<Vec<CacheEntry>>,
}

impl CacheBudget {
    const fn new() -> Self {
        Self {
            remaining_bytes: AtomicUsize::new(MAX_BYTES),
            clock: AtomicU64::new(0),
            caches: Mutex::new(Vec::new()),
        }
    }

    fn try_reserve_bytes(&self, bytes: usize) -> bool {
        self.remaining_bytes
            .fetch_update(Relaxed, Relaxed, |n| n.checked_sub(bytes))
            .is_ok()
    }

    fn evict_one(&self) -> bool {
        let clear = self
            .caches
            .lock()
            .iter()
            .filter(|entry| entry.resident_bytes.load(Relaxed) > 0)
            .min_by_key(|entry| entry.last_access.load(Relaxed))
            .map(|entry| Arc::clone(&entry.clear));
        if let Some(clear) = clear {
            clear();
            true
        } else {
            false
        }
    }

    /// Wraps a source vec in this budget and registers it for eviction.
    pub fn wrap<V>(&'static self, source: V) -> CachedVec<V>
    where
        V: TypedVec + ReadableVec<V::I, V::T> + Clone + 'static,
    {
        let access_count = Arc::new(AtomicU64::new(0));
        let last_access = Arc::new(AtomicU64::new(0));
        let resident_bytes = Arc::new(AtomicUsize::new(0));
        let cached = CachedVec::wrap_budgeted(
            source,
            self,
            access_count.clone(),
            last_access.clone(),
            resident_bytes.clone(),
        );
        let clone = cached.clone();
        self.caches.lock().push(CacheEntry {
            last_access,
            resident_bytes,
            clear: Arc::new(move || clone.clear()),
        });
        cached
    }

    /// Clears every registered vec and resets the budget.
    pub fn clear(&self) {
        let clear: Vec<_> = self
            .caches
            .lock()
            .iter()
            .map(|entry| Arc::clone(&entry.clear))
            .collect();
        for clear in clear {
            clear();
        }
    }
}

struct CacheEntry {
    last_access: Arc<AtomicU64>,
    resident_bytes: Arc<AtomicUsize>,
    clear: Arc<dyn Fn() + Send + Sync>,
}

impl CachedVecBudget for CacheBudget {
    #[inline]
    fn record_access(&self) -> u64 {
        self.clock.fetch_add(1, Relaxed) + 1
    }

    fn try_reserve(&self, access_count: u64, bytes: usize) -> bool {
        if access_count < MIN_ACCESSES {
            return false;
        }
        if bytes > MAX_BYTES {
            return false;
        }
        if self.try_reserve_bytes(bytes) {
            return true;
        }

        while self.evict_one() {
            if self.try_reserve_bytes(bytes) {
                return true;
            }
        }

        false
    }

    #[inline]
    fn release(&self, bytes: usize) {
        if bytes == 0 {
            return;
        }
        let previous = self.remaining_bytes.fetch_add(bytes, Relaxed);
        debug_assert!(previous + bytes <= MAX_BYTES);
    }
}

pub static CACHE_BUDGET: CacheBudget = CacheBudget::new();
