use std::marker::PhantomData;

/// Direct-mapped cache size. Power of 2 for fast masking.
/// 1024 entries × ~32 bytes = 32 KB (fits in L1 cache).
const CACHE_SIZE: usize = 1024;
const CACHE_MASK: usize = CACHE_SIZE - 1;

#[derive(Clone, Copy)]
struct CacheEntry<I, V>(I, I, V, bool);

/// Maps ranges of indices to values for efficient reverse lookups.
///
/// Stores first_index values in a sorted Vec and uses binary search
/// to find the value for any index. The value is derived from the position.
///
/// Includes a direct-mapped cache for O(1) floor lookups when there's locality.
pub struct RangeMap<I, V> {
    first_indexes: Vec<I>,
    cache: [CacheEntry<I, V>; CACHE_SIZE],
    _phantom: PhantomData<V>,
}

impl<I: Default + Copy, V: Default + Copy> Clone for RangeMap<I, V> {
    fn clone(&self) -> Self {
        Self {
            first_indexes: self.first_indexes.clone(),
            cache: [CacheEntry(I::default(), I::default(), V::default(), false); CACHE_SIZE],
            _phantom: PhantomData,
        }
    }
}

impl<I: Default + Copy, V: Default + Copy> From<Vec<I>> for RangeMap<I, V> {
    fn from(first_indexes: Vec<I>) -> Self {
        Self {
            first_indexes,
            cache: [CacheEntry(I::default(), I::default(), V::default(), false); CACHE_SIZE],
            _phantom: PhantomData,
        }
    }
}

impl<I: Default + Copy, V: Default + Copy> Default for RangeMap<I, V> {
    fn default() -> Self {
        Self {
            first_indexes: Vec::new(),
            cache: [CacheEntry(I::default(), I::default(), V::default(), false); CACHE_SIZE],
            _phantom: PhantomData,
        }
    }
}

impl<I: Ord + Copy + Default + Into<usize>, V: From<usize> + Copy + Default> RangeMap<I, V> {
    /// Number of ranges stored.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.first_indexes.len()
    }

    /// Truncate to `new_len` ranges and clear the cache.
    pub fn truncate(&mut self, new_len: usize) {
        self.first_indexes.truncate(new_len);
        self.clear_cache();
    }

    /// Reserve capacity for additional entries.
    pub fn reserve(&mut self, additional: usize) {
        self.first_indexes.reserve(additional);
    }

    /// Push a new first_index. Value is implicitly the current length.
    /// Must be called in order (first_index must be >= all previous).
    #[inline]
    pub fn push(&mut self, first_index: I) {
        debug_assert!(
            self.first_indexes
                .last()
                .is_none_or(|&last| first_index >= last),
            "RangeMap: first_index must be monotonically increasing"
        );
        self.first_indexes.push(first_index);
    }

    /// Returns the last pushed first_index, if any.
    #[inline]
    pub fn last_key(&self) -> Option<I> {
        self.first_indexes.last().copied()
    }

    /// Floor: returns the value (position) of the largest first_index <= given index.
    #[inline]
    pub fn get(&mut self, index: I) -> Option<V> {
        if self.first_indexes.is_empty() {
            return None;
        }

        let slot = Self::cache_slot(&index);
        let entry = &self.cache[slot];
        if entry.3 && index >= entry.0 && index < entry.1 {
            return Some(entry.2);
        }

        let pos = self.first_indexes.partition_point(|&first| first <= index);
        if pos > 0 {
            let value = V::from(pos - 1);
            if pos < self.first_indexes.len() {
                self.cache[slot] = CacheEntry(
                    self.first_indexes[pos - 1],
                    self.first_indexes[pos],
                    value,
                    true,
                );
            }
            Some(value)
        } else {
            None
        }
    }

    /// Ceil: returns the value (position) of the smallest first_index >= given index.
    #[inline]
    pub fn ceil(&self, index: I) -> Option<V> {
        if self.first_indexes.is_empty() {
            return None;
        }

        let pos = self.first_indexes.partition_point(|&first| first < index);
        if pos < self.first_indexes.len() {
            Some(V::from(pos))
        } else {
            None
        }
    }

    /// Shared (immutable) floor lookup — binary search only, no cache update.
    /// Use when you only have `&self` (e.g. read-only clones in the query layer).
    #[inline]
    pub fn get_shared(&self, index: I) -> Option<V> {
        if self.first_indexes.is_empty() {
            return None;
        }
        let pos = self.first_indexes.partition_point(|&first| first <= index);
        if pos > 0 {
            Some(V::from(pos - 1))
        } else {
            None
        }
    }

    #[inline]
    fn cache_slot(index: &I) -> usize {
        let v: usize = (*index).into();
        v & CACHE_MASK
    }

    fn clear_cache(&mut self) {
        for entry in self.cache.iter_mut() {
            entry.3 = false;
        }
    }
}
