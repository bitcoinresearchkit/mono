use std::{marker::PhantomData, sync::Arc};

use parking_lot::RwLock;

mod any_vec;
mod readable;
mod typed;

use crate::{
    CompressedIoSource, CompressedMmapSource, CompressedRangeCursor, ReadOnlyBaseVec, VecIndex,
    VecValue,
};

use super::{CompressionStrategy, Pages, ReadWriteCompressedVec};

/// Lean read-only view of a compressed vector (~48 bytes).
///
/// Carries only the fields needed for disk reads: region, shared length,
/// name/header metadata, and the pages index. No pushed buffer, no rollback state.
///
/// Created via `ReadWriteCompressedVec::read_only_clone`.
#[derive(Debug, Clone)]
pub struct ReadOnlyCompressedVec<I, T, S> {
    pub(super) base: ReadOnlyBaseVec<I, T>,
    pub(super) pages: Arc<RwLock<Pages>>,
    _strategy: PhantomData<S>,
}

impl<I, T, S> ReadOnlyCompressedVec<I, T, S> {
    pub(crate) fn new(base: ReadOnlyBaseVec<I, T>, pages: Arc<RwLock<Pages>>) -> Self {
        Self {
            base,
            pages,
            _strategy: PhantomData,
        }
    }
}

impl<I, T, S> ReadOnlyCompressedVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    /// Creates a forward cursor over a bounded persisted range.
    #[inline]
    pub fn range_cursor_at(&self, from: usize, to: usize) -> CompressedRangeCursor<'_, I, T, S> {
        CompressedRangeCursor::new(self.base.region(), &self.pages, self.base.len(), from, to)
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
        let mmap = ReadWriteCompressedVec::<I, T, S>::prefers_mmap(
            self.base.region(),
            &self.pages,
            from,
            to,
        );
        if mmap {
            CompressedMmapSource::<I, T, S>::new_from_parts(
                self.base.region(),
                &self.pages,
                len,
                from,
                to,
            )
            .fold(init, f)
        } else {
            CompressedIoSource::<I, T, S>::new_from_parts(
                self.base.region(),
                &self.pages,
                len,
                from,
                to,
            )
            .fold(init, f)
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
        let mmap = ReadWriteCompressedVec::<I, T, S>::prefers_mmap(
            self.base.region(),
            &self.pages,
            from,
            to,
        );
        if mmap {
            CompressedMmapSource::<I, T, S>::new_from_parts(
                self.base.region(),
                &self.pages,
                len,
                from,
                to,
            )
            .try_fold(init, f)
        } else {
            CompressedIoSource::<I, T, S>::new_from_parts(
                self.base.region(),
                &self.pages,
                len,
                from,
                to,
            )
            .try_fold(init, f)
        }
    }
}
