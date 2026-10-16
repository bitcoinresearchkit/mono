use super::{super::CompressionStrategy, ReadWriteCompressedVec};
use crate::{
    VecIndex, VecValue,
    cache::{Cache, CachePolicy},
    variants::{compressed::inner::persisted::CompressedPersisted, stored_reads::StoredReads},
};

impl<I, T, S, C: CachePolicy> StoredReads<T> for ReadWriteCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    type Persisted<'a>
        = CompressedPersisted<'a, I, T, S>
    where
        Self: 'a;

    const CHUNK_SIZE: usize = Self::PER_PAGE;

    #[inline(always)]
    fn cache(&self) -> Option<&Cache<T>> {
        C::cache(&self.cache)
    }

    #[inline(always)]
    fn layers(&self) -> (usize, &[T]) {
        (self.base.stored_len(), self.base.pushed())
    }

    #[inline(always)]
    fn persisted(&self) -> Self::Persisted<'_> {
        CompressedPersisted::new(self.base.region(), &self.pages)
    }
}

impl_stored_readable!(ReadWriteCompressedVec, CompressionStrategy);
