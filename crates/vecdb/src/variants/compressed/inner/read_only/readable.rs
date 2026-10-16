use super::{
    super::{CompressionStrategy, ReadWriteCompressedVec},
    ReadOnlyCompressedVec,
};
use crate::{
    VecIndex, VecValue,
    cache::{Cache, CachePolicy},
    variants::{compressed::inner::persisted::CompressedPersisted, stored_reads::StoredReads},
};

impl<I, T, S, C: CachePolicy> StoredReads<T> for ReadOnlyCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    type Persisted<'a>
        = CompressedPersisted<'a, I, T, S>
    where
        Self: 'a;

    const CHUNK_SIZE: usize = ReadWriteCompressedVec::<I, T, S>::PER_PAGE;

    #[inline(always)]
    fn cache(&self) -> Option<&Cache<T>> {
        C::cache(&self.cache)
    }

    #[inline(always)]
    fn layers(&self) -> (usize, &[T]) {
        (self.base.len(), &[])
    }

    #[inline(always)]
    fn persisted(&self) -> Self::Persisted<'_> {
        CompressedPersisted::new(self.base.region(), &self.pages)
    }
}

impl_stored_readable!(ReadOnlyCompressedVec, CompressionStrategy);
