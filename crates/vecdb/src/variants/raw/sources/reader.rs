use std::marker::PhantomData;

use rawdb::{Reader, Region};

use crate::{AnyStoredVec, HEADER_OFFSET, ReadOnlyRawVec, VecIndex, VecValue};

use super::super::{RawStrategy, ReadWriteRawVec};

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
    _reader: Reader,
    data: *const u8,
    stored_len: usize,
    _marker: PhantomData<(I, T, S)>,
}

/// Forward cursor over a raw vector reader.
///
/// Unlike [`crate::Cursor`], this reads values directly from the existing mmap
/// and does not allocate or copy through a staging buffer. Like [`VecReader`],
/// it only sees persisted values.
pub struct VecReaderCursor<I, T, S> {
    reader: VecReader<I, T, S>,
    pos: usize,
}

unsafe impl<I: Send, T: Send, S: Send> Send for VecReader<I, T, S> {}
unsafe impl<I: Sync, T: Sync, S: Sync> Sync for VecReader<I, T, S> {}

impl<I, T, S> VecReader<I, T, S>
where
    T: VecValue,
    S: RawStrategy<T>,
{
    const SIZE_OF_T: usize = size_of::<T>();

    pub(crate) fn from_region(region: &Region, stored_len: usize) -> Self {
        let reader = region.create_reader();
        let slice = reader.prefixed(HEADER_OFFSET);
        let ptr = slice.as_ptr();

        Self {
            _reader: reader,
            data: ptr,
            stored_len,
            _marker: PhantomData,
        }
    }

    pub(crate) fn from_read_write(vec: &ReadWriteRawVec<I, T, S>) -> Self
    where
        I: VecIndex,
    {
        Self::from_region(vec.region(), vec.stored_len())
    }

    pub(crate) fn from_read_only(vec: &ReadOnlyRawVec<I, T, S>) -> Self
    where
        I: VecIndex,
    {
        Self::from_region(vec.region(), vec.stored_len())
    }

    /// Returns the value at typed `index`.
    ///
    /// # Panics
    /// Panics if `index >= len()`.
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
    /// Panics if `index >= len()`.
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
    pub fn len(&self) -> usize {
        self.stored_len
    }

    /// Returns `true` if the reader is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.stored_len == 0
    }

    #[inline(always)]
    #[cfg(feature = "zerocopy")]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        // SAFETY: `data` points to `stored_len * SIZE_OF_T` bytes and `_reader`
        // keeps the mmap generation containing them alive for this borrow.
        unsafe { std::slice::from_raw_parts(self.data, self.stored_len * Self::SIZE_OF_T) }
    }

    /// Creates an allocation-free cursor over the persisted values.
    #[inline]
    pub fn cursor(self) -> VecReaderCursor<I, T, S> {
        VecReaderCursor {
            reader: self,
            pos: 0,
        }
    }
}

impl<I, T, S> VecReaderCursor<I, T, S>
where
    T: VecValue,
    S: RawStrategy<T>,
{
    /// Returns the current absolute position.
    #[inline(always)]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Returns the number of values remaining.
    #[inline(always)]
    pub fn remaining(&self) -> usize {
        self.reader.len().saturating_sub(self.pos)
    }

    /// Advances the position by `n` without reading.
    #[inline(always)]
    pub fn advance(&mut self, n: usize) {
        self.pos = self.pos.saturating_add(n).min(self.reader.len());
    }

    /// Returns the value at absolute `index` without changing the position.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<T> {
        self.reader.try_get_at(index)
    }

    /// Returns the next value and advances the position.
    #[inline(always)]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<T> {
        let value = self.reader.try_get_at(self.pos)?;
        self.pos += 1;
        Some(value)
    }

    /// Folds over the next `n` values and advances the position.
    #[inline]
    pub fn fold<B>(&mut self, n: usize, mut init: B, mut f: impl FnMut(B, T) -> B) -> B {
        let end = self.pos.saturating_add(n).min(self.reader.len());
        while self.pos < end {
            init = f(init, self.reader.get_at(self.pos));
            self.pos += 1;
        }
        init
    }

    /// Calls `f` for each of the next `n` values and advances the position.
    #[inline]
    pub fn for_each(&mut self, n: usize, mut f: impl FnMut(T)) {
        self.fold(n, (), |(), value| f(value));
    }
}
