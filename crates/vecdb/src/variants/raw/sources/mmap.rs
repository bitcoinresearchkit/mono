use std::marker::PhantomData;

use rawdb::Reader;

use rawdb::Region;

use crate::{AnyStoredVec, HEADER_OFFSET, VecIndex, VecValue};

use super::super::{RawStrategy, ReadWriteRawVec};

/// Read-only mmap-backed source over a raw (uncompressed) vector.
///
/// Only sees **stored** (persisted) values — pushed but unflushed values
/// are not visible. Created with a range and consumed by fold/try_fold/for_each.
///
/// The data slice is computed once at construction time (matching Reader's
/// own `transmute` pattern), so fold/for_each are direct slice operations.
pub struct RawMmapSource<I, T, S> {
    // SAFETY: Field order matters. `_reader` keeps the mmap guard alive.
    // `data` is a pointer into that mmap. `_reader` must outlive `data`,
    // which it does because `data` is a raw pointer with no destructor.
    _reader: Reader,
    data: *const u8,
    pos: usize,
    end: usize,
    _marker: PhantomData<(I, T, S)>,
}

// SAFETY: RawMmapSource is read-only. The mmap data it points to is shared
// immutable memory protected by Reader's RwLockReadGuard, which is Sync.
unsafe impl<I: Send, T: Send, S: Send> Send for RawMmapSource<I, T, S> {}
unsafe impl<I: Sync, T: Sync, S: Sync> Sync for RawMmapSource<I, T, S> {}

impl<I, T, S> RawMmapSource<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    const SIZE_OF_T: usize = size_of::<T>();

    pub(crate) fn new(vec: &ReadWriteRawVec<I, T, S>, from: usize, to: usize) -> Self {
        Self::new_from_parts(vec.region(), vec.stored_len(), from, to)
    }

    pub(crate) fn new_from_parts(
        region: &Region,
        stored_len: usize,
        from: usize,
        to: usize,
    ) -> Self {
        let reader = region.create_reader();
        let from = from.min(stored_len);
        let to = to.min(stored_len);
        let slice = reader.prefixed(HEADER_OFFSET);
        let ptr = slice.as_ptr();

        Self {
            _reader: reader,
            data: ptr,
            pos: from,
            end: to,
            _marker: PhantomData,
        }
    }

    pub(crate) fn byte_window(&self) -> (*const u8, usize) {
        let byte_position = self.pos * Self::SIZE_OF_T;
        let byte_len = (self.end - self.pos) * Self::SIZE_OF_T;
        (unsafe { self.data.add(byte_position) }, byte_len)
    }

    /// Fold all elements in the range — tight pointer loop.
    #[inline(always)]
    pub(crate) fn fold<B, F: FnMut(B, T) -> B>(self, init: B, mut f: F) -> B {
        let ptr = self.data;
        let mut byte_off = self.pos * Self::SIZE_OF_T;
        let end_byte = self.end * Self::SIZE_OF_T;
        let mut acc = init;
        while byte_off < end_byte {
            acc = f(acc, unsafe { S::read_from_ptr(ptr, byte_off) });
            byte_off += Self::SIZE_OF_T;
        }
        acc
    }

    /// Fallible fold with early exit on error.
    #[inline(always)]
    pub(crate) fn try_fold<B, E, F: FnMut(B, T) -> std::result::Result<B, E>>(
        self,
        init: B,
        mut f: F,
    ) -> std::result::Result<B, E> {
        let ptr = self.data;
        let mut byte_off = self.pos * Self::SIZE_OF_T;
        let end_byte = self.end * Self::SIZE_OF_T;
        let mut acc = init;
        while byte_off < end_byte {
            acc = f(acc, unsafe { S::read_from_ptr(ptr, byte_off) })?;
            byte_off += Self::SIZE_OF_T;
        }
        Ok(acc)
    }
}
