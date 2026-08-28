use std::sync::{
    Arc,
    atomic::{AtomicU64, AtomicUsize, Ordering::Relaxed},
};

use parking_lot::Mutex;
use vecdb::{
    CachedVec, CachedVecBudget, ReadableBoxedVec, ReadableVec, TypedVec, VecIndex, VecValue,
};

const MAX_BYTES: usize = 2 * 1024 * 1024 * 1024;

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
        let invalidate = self
            .caches
            .lock()
            .iter()
            .filter(|entry| entry.resident_bytes.load(Relaxed) > 0)
            .min_by_key(|entry| entry.last_access.load(Relaxed))
            .map(|entry| Arc::clone(&entry.invalidate));
        if let Some(invalidate) = invalidate {
            invalidate();
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
        let last_access = Arc::new(AtomicU64::new(0));
        let resident_bytes = Arc::new(AtomicUsize::new(0));
        let cached =
            CachedVec::wrap_budgeted(source, self, last_access.clone(), resident_bytes.clone());
        let invalidated = cached.clone();
        self.caches.lock().push(CacheEntry {
            last_access,
            resident_bytes,
            invalidate: Arc::new(move || invalidated.invalidate()),
        });
        cached
    }

    /// Adds this budget's cache unless the type-erased source is already cached.
    pub fn wrap_boxed<I, T>(&'static self, source: ReadableBoxedVec<I, T>) -> ReadableBoxedVec<I, T>
    where
        I: VecIndex,
        T: VecValue,
    {
        if source.has_cache_layer() {
            source
        } else {
            ReadableBoxedVec::new(self.wrap(source))
        }
    }

    /// Invalidates every registered vec.
    pub fn invalidate(&self) {
        let invalidate: Vec<_> = self
            .caches
            .lock()
            .iter()
            .map(|entry| Arc::clone(&entry.invalidate))
            .collect();
        for invalidate in invalidate {
            invalidate();
        }
    }
}

struct CacheEntry {
    last_access: Arc<AtomicU64>,
    resident_bytes: Arc<AtomicUsize>,
    invalidate: Arc<dyn Fn() + Send + Sync>,
}

impl CachedVecBudget for CacheBudget {
    #[inline]
    fn record_access(&self) -> u64 {
        self.clock.fetch_add(1, Relaxed) + 1
    }

    fn try_reserve(&self, bytes: usize) -> bool {
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
