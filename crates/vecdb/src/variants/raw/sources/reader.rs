use std::marker::PhantomData;

use rawdb::{Reader, Region};

use super::super::{RawStrategy, ReadWriteRawVec};
use crate::{AnyStoredVec, HEADER_OFFSET, ReadOnlyRawVec, VecIndex, VecValue, cache::CachePolicy};

/// Read-only random-access handle into a raw vector's stored data.
///
/// Created via `raw_vec.reader()` (available on BytesVec/ZeroCopyVec via Deref).
/// Provides O(1) point reads directly from the memory-mapped file.
///
/// Only sees **stored** (persisted) values, not pushed values. A
/// [`MutableVec`](crate::MutableVec) applies its update and hole overlays around
/// this reader.
///
/// The reader holds the current mmap generation alive. Drop long-lived readers
/// before writes that may grow the database and remap the file.
pub struct VecReader<I, T, S> {
    reader: Reader,
    data: *const u8,
    stored_len: usize,
    _marker: PhantomData<(I, T, S)>,
}

impl<I, T, S> VecReader<I, T, S>
where
    T: VecValue,
    S: RawStrategy<T>,
{
    const SIZE_OF_T: usize = size_of::<T>();

    /// Hints the OS to start reading the pages holding the value at typed `index`
    /// ([`Reader::will_need`]). No-op past `stored_len()`.
    #[inline]
    pub fn prefetch(&self, index: I)
    where
        I: VecIndex,
    {
        let index = index.to_usize();
        if index < self.stored_len {
            self.reader
                .will_need(HEADER_OFFSET + index * Self::SIZE_OF_T, Self::SIZE_OF_T);
        }
    }

    /// Returns the value at typed `index`.
    ///
    /// # Panics
    /// Panics if `index >= stored_len()`.
    #[inline(always)]
    pub fn get(&self, index: I) -> T
    where
        I: VecIndex,
    {
        self.get_at(index.to_usize())
    }

    /// Returns the value at raw `index`.
    ///
    /// # Panics
    /// Panics if `index >= stored_len()`.
    #[inline(always)]
    pub fn get_at(&self, index: usize) -> T {
        assert!(
            index < self.stored_len,
            "index {index} out of bounds (len {})",
            self.stored_len
        );
        // SAFETY: index < stored_len guarantees offset + SIZE_OF_T <= data_len
        unsafe { S::read_from_ptr(self.data, index * Self::SIZE_OF_T) }
    }

    /// Returns the value at typed `index`, or `None` if out of bounds.
    #[inline(always)]
    pub fn try_get(&self, index: I) -> Option<T>
    where
        I: VecIndex,
    {
        self.try_get_at(index.to_usize())
    }

    /// Returns the value at raw `index`, or `None` if out of bounds.
    #[inline(always)]
    pub fn try_get_at(&self, index: usize) -> Option<T> {
        if index >= self.stored_len {
            return None;
        }
        // SAFETY: index < stored_len guarantees offset + SIZE_OF_T <= data_len
        Some(unsafe { S::read_from_ptr(self.data, index * Self::SIZE_OF_T) })
    }

    /// Returns the number of stored values.
    #[inline(always)]
    pub fn stored_len(&self) -> usize {
        self.stored_len
    }

    fn from_region(region: &Region, stored_len: usize) -> Self {
        let reader = region.create_reader();
        let slice = reader.read_from(HEADER_OFFSET);
        let ptr = slice.as_ptr();

        Self {
            reader,
            data: ptr,
            stored_len,
            _marker: PhantomData,
        }
    }
    pub(crate) fn from_read_write<C: CachePolicy>(vec: &ReadWriteRawVec<I, T, S, C>) -> Self
    where
        I: VecIndex,
    {
        Self::from_region(vec.region(), vec.stored_len())
    }
    pub(crate) fn from_read_only<C: CachePolicy>(vec: &ReadOnlyRawVec<I, T, S, C>) -> Self
    where
        I: VecIndex,
    {
        Self::from_region(vec.region(), vec.stored_len())
    }
}

unsafe impl<I: Send, T: Send, S: Send> Send for VecReader<I, T, S> {}

unsafe impl<I: Sync, T: Sync, S: Sync> Sync for VecReader<I, T, S> {}
