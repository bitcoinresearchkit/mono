use std::{collections::VecDeque, hash::Hash, sync::Arc};

use parking_lot::RwLock;
use rustc_hash::FxHashMap;

use crate::RepresentationId;

use super::CachedJson;

struct CacheEntry<S> {
    source: S,
    json: CachedJson,
}

struct Entries<K, S> {
    values: FxHashMap<K, CacheEntry<S>>,
    order: VecDeque<K>,
    bytes: usize,
}

impl<K, S> Default for Entries<K, S> {
    fn default() -> Self {
        Self {
            values: FxHashMap::default(),
            order: VecDeque::default(),
            bytes: 0,
        }
    }
}

/// Exact JSON representations with shared hard entry and byte bounds.
pub(crate) struct BoundedJsonCache<K, S> {
    entries: RwLock<Entries<K, S>>,
    max_entries: usize,
    max_bytes: usize,
}

impl<K, S> BoundedJsonCache<K, S>
where
    K: Clone + Eq + Hash,
    S: PartialEq,
{
    pub(crate) fn new(max_entries: usize, max_bytes: usize) -> Self {
        debug_assert!(max_entries > 0);
        Self {
            entries: RwLock::default(),
            max_entries,
            max_bytes,
        }
    }

    pub(crate) fn current(&self, key: &K, source: &S) -> Option<(Arc<[u8]>, RepresentationId)> {
        let entries = self.entries.try_read()?;
        let entry = entries.values.get(key)?;
        (&entry.source == source).then(|| entry.json.value())
    }

    pub(crate) fn insert(
        &self,
        key: K,
        source: S,
        json: CachedJson,
    ) -> (Arc<[u8]>, RepresentationId) {
        let value = json.value();
        let bytes = json.len();
        if bytes > self.max_bytes {
            return value;
        }

        let mut entries = self.entries.write();
        if let Some(entry) = entries.values.get(&key)
            && entry.source == source
        {
            return entry.json.value();
        }
        if let Some(previous) = entries.values.remove(&key) {
            entries.bytes -= previous.json.len();
            entries.order.retain(|candidate| candidate != &key);
        }
        while entries.values.len() >= self.max_entries || entries.bytes + bytes > self.max_bytes {
            let Some(oldest) = entries.order.pop_front() else {
                break;
            };
            if let Some(removed) = entries.values.remove(&oldest) {
                entries.bytes -= removed.json.len();
            }
        }

        entries.bytes += bytes;
        entries.order.push_back(key.clone());
        entries.values.insert(key, CacheEntry { source, json });
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_enforces_entry_and_byte_bounds() {
        let cache = BoundedJsonCache::new(2, 10);
        cache.insert(1, 1, CachedJson::from_slice(b"a"));
        cache.insert(2, 2, CachedJson::from_slice(b"b"));
        cache.insert(3, 3, CachedJson::from_slice(b"c"));
        assert_eq!(cache.entries.read().values.len(), 2);
        assert!(!cache.entries.read().values.contains_key(&1));

        cache.insert(4, 4, CachedJson::from_slice(b"123456789"));
        let entries = cache.entries.read();
        assert_eq!(entries.values.len(), 2);
        assert_eq!(entries.bytes, 10);
        assert!(entries.values.contains_key(&4));
        drop(entries);

        cache.insert(5, 5, CachedJson::from_slice(b"12345678901"));
        let entries = cache.entries.read();
        assert_eq!(entries.values.len(), 2);
        assert_eq!(entries.bytes, 10);
        assert!(!entries.values.contains_key(&5));
    }
}
