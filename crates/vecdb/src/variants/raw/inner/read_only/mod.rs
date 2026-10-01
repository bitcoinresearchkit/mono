use std::result::Result;

use rawdb::Region;

use super::{RawStrategy, ReadOnlyRawVec};
use crate::{
    HEADER_OFFSET, RawIoSource, RawMmapSource, VecIndex, VecReader, VecValue, cache::CachePolicy,
};

pub mod any_vec;
pub mod readable;
pub mod typed;

impl<I, T, S, C: CachePolicy> ReadOnlyRawVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    pub fn reader(&self) -> VecReader<I, T, S> {
        VecReader::from_read_only(self)
    }

    pub(crate) fn region(&self) -> &Region {
        self.base.region()
    }
    pub(crate) fn stored_len(&self) -> usize {
        self.base.stored_len()
    }
    #[inline(always)]
    fn fold_source<B, F: FnMut(B, T) -> B>(
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
    fn try_fold_source<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        len: usize,
        init: B,
        f: F,
    ) -> Result<B, E> {
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
