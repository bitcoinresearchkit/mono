use crate::{
    BytesStrategy, BytesVec, BytesVecReader, MutableVec, OverflowVecValue, ReadOnlyMutableVec,
    ReadOnlyRawVec, VecIndex, unlikely,
};

/// Reusable random-access reader for an [`OverflowVec`](crate::OverflowVec).
pub struct OverflowVecReader<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    compact: BytesVecReader<I, T::Compact>,
    overflow: BytesVecReader<usize, T>,
}

impl<I, T> OverflowVecReader<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    #[doc(hidden)]
    pub fn new(
        compact: &MutableVec<BytesVec<I, T::Compact>>,
        overflow: &MutableVec<BytesVec<usize, T>>,
    ) -> Self {
        Self {
            compact: compact.reader(),
            overflow: overflow.reader(),
        }
    }

    #[doc(hidden)]
    pub fn from_read_only(
        compact: &ReadOnlyMutableVec<ReadOnlyRawVec<I, T::Compact, BytesStrategy<T::Compact>>>,
        overflow: &ReadOnlyMutableVec<ReadOnlyRawVec<usize, T, BytesStrategy<T>>>,
    ) -> Self {
        Self {
            compact: BytesVecReader::new(compact.reader()),
            overflow: BytesVecReader::new(overflow.reader()),
        }
    }

    #[inline(always)]
    fn decode(&self, compact: T::Compact) -> T {
        let overflow_index = T::overflow_index(compact);
        if unlikely(overflow_index.is_some()) {
            self.overflow.get_at(overflow_index.unwrap())
        } else {
            T::from_compact(compact)
        }
    }

    /// Returns the persisted value at `index`.
    ///
    /// # Panics
    /// Panics if `index >= len()`.
    #[inline(always)]
    pub fn get(&self, index: I) -> T {
        self.get_at(index.to_usize())
    }

    /// Returns the persisted value at raw `index`.
    ///
    /// # Panics
    /// Panics if `index >= len()`.
    #[inline(always)]
    pub fn get_at(&self, index: usize) -> T {
        self.decode(self.compact.get_at(index))
    }

    /// Returns the persisted value at `index`, or `None` if out of bounds.
    #[inline(always)]
    pub fn try_get(&self, index: I) -> Option<T> {
        self.try_get_at(index.to_usize())
    }

    /// Returns the persisted value at raw `index`, or `None` if out of bounds.
    #[inline(always)]
    pub fn try_get_at(&self, index: usize) -> Option<T> {
        self.compact
            .try_get_at(index)
            .map(|compact| self.decode(compact))
    }

    /// Returns the number of persisted values.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.compact.len()
    }

    /// Returns whether there are no persisted values.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.compact.is_empty()
    }

    #[inline(always)]
    #[doc(hidden)]
    pub fn compact(
        &self,
        vec: &MutableVec<BytesVec<I, T::Compact>>,
        index: I,
    ) -> Option<T::Compact> {
        vec.get_with_reader(index, &self.compact)
    }

    #[inline(always)]
    #[doc(hidden)]
    pub fn overflow(&self, vec: &MutableVec<BytesVec<usize, T>>, index: usize) -> Option<T> {
        vec.get_with_reader_at(index, &self.overflow)
    }
}
