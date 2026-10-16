use std::{marker::PhantomData, sync::Arc};

use super::{super::CompressionStrategy, ReadOnlyCompressedVec, ReadWriteCompressedVec};
use crate::{
    ImportOptions, ImportableVec, ReadableBoxedVec, ReadableCloneableVec, Result, StoredVec,
    VecIndex, VecValue, cache::CachePolicy,
};

impl<I, T, S, C: CachePolicy> ImportableVec for ReadWriteCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    fn import_with(options: ImportOptions) -> Result<Self> {
        Self::import_resetting(options)
    }
}

impl<I, T, S, C: CachePolicy> StoredVec for ReadWriteCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    type ReadOnly = ReadOnlyCompressedVec<I, T, S, C>;

    #[inline]
    fn read_only_clone(&self) -> Self::ReadOnly {
        ReadOnlyCompressedVec {
            base: self.base.read_only_base(),
            cache: self.cache.clone(),
            pages: Arc::clone(&self.pages),
            _strategy: PhantomData,
        }
    }
}

impl<I, T, S, C: CachePolicy> ReadableCloneableVec<I, T> for ReadWriteCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    #[inline]
    fn read_only_boxed_clone(&self) -> ReadableBoxedVec<I, T> {
        ReadableBoxedVec::from_read_only(StoredVec::read_only_clone(self))
    }
}
