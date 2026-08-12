use std::sync::{
    Arc,
    atomic::{AtomicU64, AtomicUsize, Ordering::Relaxed},
};

use parking_lot::Mutex;
use vecdb::{CachedVec, CachedVecBudget, ReadableVec, TypedVec};

const MAX_CACHED: usize = 256;
const MIN_ACCESSES: u64 = 2;

pub struct CacheBudget {
    remaining: AtomicUsize,
    caches: Mutex<Vec<CacheEntry>>,
}

impl CacheBudget {
    const fn new() -> Self {
        Self {
            remaining: AtomicUsize::new(MAX_CACHED),
            caches: Mutex::new(Vec::new()),
        }
    }

    fn try_decrement(&self) -> bool {
        self.remaining
            .fetch_update(Relaxed, Relaxed, |n| if n > 0 { Some(n - 1) } else { None })
            .is_ok()
    }

    fn evict_less_popular_than(&self, threshold: u64) -> bool {
        let caches = self.caches.lock();
        if let Some((index, _)) = caches
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                let count = entry.access_count.load(Relaxed);
                count >= MIN_ACCESSES && count < threshold
            })
            .min_by_key(|(_, entry)| entry.access_count.load(Relaxed))
        {
            (caches[index].clear)();
            self.remaining.fetch_add(1, Relaxed);
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
        let cached = CachedVec::wrap_budgeted(source, self, access_count.clone());
        let clone = cached.clone();
        self.caches.lock().push(CacheEntry {
            access_count,
            clear: Box::new(move || clone.clear()),
        });
        cached
    }

    /// Clears every registered vec and resets the budget.
    pub fn clear(&self) {
        for entry in self.caches.lock().iter() {
            (entry.clear)();
        }
        self.remaining.store(MAX_CACHED, Relaxed);
    }
}

struct CacheEntry {
    access_count: Arc<AtomicU64>,
    clear: Box<dyn Fn() + Send + Sync>,
}

impl CachedVecBudget for CacheBudget {
    fn try_reserve(&self, access_count: u64) -> bool {
        if access_count < MIN_ACCESSES {
            return false;
        }
        if self.try_decrement() {
            return true;
        }
        if self.evict_less_popular_than(access_count) {
            self.try_decrement()
        } else {
            false
        }
    }
}

pub static CACHE_BUDGET: CacheBudget = CacheBudget::new();
