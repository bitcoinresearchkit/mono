use std::result::Result;

use super::{CompressionStrategy, ReadOnlyCompressedVec, ReadWriteCompressedVec};
use crate::{CompressedIoSource, CompressedMmapSource, VecIndex, VecValue, cache::CachePolicy};

pub mod any_vec;
pub mod readable;
pub mod typed;

impl<I, T, S, C: CachePolicy> ReadOnlyCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    #[inline(always)]
    fn fold_source<B, F: FnMut(B, T) -> B>(
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
    fn try_fold_source<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        len: usize,
        init: B,
        f: F,
    ) -> Result<B, E> {
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
