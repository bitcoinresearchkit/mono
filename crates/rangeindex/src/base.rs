use std::marker::PhantomData;

use crate::{Pages, RangeMapCursor};

/// Maps ranges of indices to values for efficient reverse lookups.
///
/// Stores first_index values in sorted pages and uses binary search
/// to find the value for any index. The value is derived from the position.
pub struct RangeMap<I, V> {
    first_indexes: Pages<I>,
    value: PhantomData<fn() -> V>,
}

impl<I, V> Clone for RangeMap<I, V> {
    fn clone(&self) -> Self {
        Self {
            first_indexes: self.first_indexes.clone(),
            value: PhantomData,
        }
    }
}

impl<I, V> Default for RangeMap<I, V> {
    fn default() -> Self {
        Self {
            first_indexes: Pages::default(),
            value: PhantomData,
        }
    }
}

impl<I, V> RangeMap<I, V> {
    /// Number of ranges stored.
    #[allow(clippy::len_without_is_empty)]
    #[inline]
    pub fn len(&self) -> usize {
        self.first_indexes.len()
    }

    /// Forward mapping: the first index of each range, in value order.
    #[inline]
    pub fn starts(&self) -> &Pages<I> {
        &self.first_indexes
    }
}

impl<I: Ord + Copy, V> RangeMap<I, V> {
    /// Truncate to `new_len` ranges.
    pub fn truncate(&mut self, new_len: usize) {
        self.first_indexes.truncate(new_len);
    }

    /// Extend an ordered suffix without copying the unchanged prefix.
    pub fn extend(&mut self, values: impl IntoIterator<Item = I>) {
        for value in values {
            debug_assert!(
                self.first_indexes.last().is_none_or(|&last| value >= last),
                "RangeMap: first_index must be monotonically increasing"
            );
            self.first_indexes.push(value);
        }
    }
}

impl<I: Ord + Copy, V: From<usize>> RangeMap<I, V> {
    /// Request-local floor lookup hint. The shared borrow prevents mutations
    /// from invalidating a remembered interval.
    pub fn cursor(&self) -> RangeMapCursor<'_, I, V> {
        RangeMapCursor::new(self)
    }

    /// Floor lookup by binary search.
    #[inline]
    pub fn get_shared(&self, index: I) -> Option<V> {
        self.first_indexes
            .partition_point(|&first| first <= index)
            .checked_sub(1)
            .map(V::from)
    }
}
