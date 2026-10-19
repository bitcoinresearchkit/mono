use std::{marker::PhantomData, ops::Range};

use crate::{Pages, RangeMapCursor};

/// Sorted range starts in shared pages: an index maps to `V::from(position)` of the last start
/// at or below it.
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

impl<I: Copy + Into<usize>, V> RangeMap<I, V> {
    /// Range `index`'s indexes below `len`: from its first index up to the next range's (to
    /// `len` for the last range); empty when it starts at or past `len`.
    #[inline]
    pub fn range(&self, index: usize, len: usize) -> Range<usize> {
        let end = self
            .first_indexes
            .get(index + 1)
            .map_or(len, |&next| next.into().min(len));
        self.first_indexes[index].into().min(end)..end
    }
}

impl<I: Ord + Copy, V> RangeMap<I, V> {
    /// Truncate to `new_len` ranges.
    pub fn truncate(&mut self, new_len: usize) {
        self.first_indexes.truncate(new_len);
    }

    /// Extend an ordered suffix without copying the unchanged prefix.
    pub fn extend(&mut self, starts: impl IntoIterator<Item = I>) {
        for start in starts {
            debug_assert!(
                self.first_indexes.last().is_none_or(|&last| start >= last),
                "RangeMap starts must be nondecreasing"
            );
            self.first_indexes.push(start);
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
    pub fn get(&self, index: I) -> Option<V> {
        self.first_indexes
            .partition_point(|&first| first <= index)
            .checked_sub(1)
            .map(V::from)
    }
}
