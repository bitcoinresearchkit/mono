use std::marker::PhantomData;

mod any_vec;
mod readable;
mod typed;

use crate::{
    Error, HEADER_OFFSET, RawIoSource, RawMmapSource, ReadOnlyBaseVec, Result, Stamp, VecIndex,
    VecReader, VecValue,
};

use super::RawStrategy;

/// Lean read-only view of a raw vector (~40 bytes).
///
/// Carries only the fields needed for disk reads: region, shared length,
/// name/header metadata. No holes, no updated map, no pushed buffer,
/// no rollback state.
///
/// Created via `ReadWriteRawVec::read_only_clone`.
#[derive(Debug, Clone)]
pub struct ReadOnlyRawVec<I, T, S> {
    pub(super) base: ReadOnlyBaseVec<I, T>,
    _strategy: PhantomData<S>,
}

impl<I, T, S> ReadOnlyRawVec<I, T, S> {
    pub(crate) fn new(base: ReadOnlyBaseVec<I, T>) -> Self {
        Self {
            base,
            _strategy: PhantomData,
        }
    }
}

impl<I, T, S> ReadOnlyRawVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    pub(crate) fn region(&self) -> &rawdb::Region {
        self.base.region()
    }

    pub(crate) fn stored_len(&self) -> usize {
        self.base.stored_len()
    }

    #[inline]
    pub fn stamp(&self) -> Stamp {
        self.base.header().stamp()
    }

    pub fn reader(&self) -> VecReader<I, T, S> {
        VecReader::from_read_only(self)
    }

    #[inline]
    pub fn read_at_once(&self, index: usize) -> Result<T> {
        let len = self.base.len();
        if index >= len {
            return Err(Error::IndexTooHigh {
                index,
                len,
                name: self.base.name().to_string(),
            });
        }

        Ok(self.base.region().with_read_bytes(|bytes| unsafe {
            S::read_from_ptr(bytes.as_ptr().add(HEADER_OFFSET), index * size_of::<T>())
        }))
    }

    #[inline]
    pub fn read_once(&self, index: I) -> Result<T> {
        self.read_at_once(index.to_usize())
    }

    #[inline(always)]
    pub(super) fn fold_source<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        len: usize,
        init: B,
        f: F,
    ) -> B {
        let offset = HEADER_OFFSET + from * size_of::<T>();
        let bytes = (to - from) * size_of::<T>();
        if self.base.region().prefers_mmap(offset, bytes) {
            RawMmapSource::<I, T, S>::new_from_parts(self.base.region(), len, from, to)
                .fold(init, f)
        } else {
            RawIoSource::<I, T, S>::new_from_parts(self.base.region(), len, from, to).fold(init, f)
        }
    }

    #[inline(always)]
    pub(super) fn try_fold_source<B, E, F: FnMut(B, T) -> std::result::Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        len: usize,
        init: B,
        f: F,
    ) -> std::result::Result<B, E> {
        let offset = HEADER_OFFSET + from * size_of::<T>();
        let bytes = (to - from) * size_of::<T>();
        if self.base.region().prefers_mmap(offset, bytes) {
            RawMmapSource::<I, T, S>::new_from_parts(self.base.region(), len, from, to)
                .try_fold(init, f)
        } else {
            RawIoSource::<I, T, S>::new_from_parts(self.base.region(), len, from, to)
                .try_fold(init, f)
        }
    }
}
