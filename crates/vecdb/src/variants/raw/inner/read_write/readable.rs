use super::{super::RawStrategy, ReadWriteRawVec};
use crate::{
    READ_CHUNK_SIZE, VecIndex, VecValue,
    cache::{Cache, CachePolicy},
    variants::{raw::inner::persisted::RawPersisted, stored_reads::StoredReads},
};

impl<I, T, S, C: CachePolicy> StoredReads<T> for ReadWriteRawVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    type Persisted<'a>
        = RawPersisted<'a, T, S>
    where
        Self: 'a;

    const PUSHED_CHUNK: usize = READ_CHUNK_SIZE;

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
        RawPersisted::new(&self.base.region)
    }
}

impl_stored_readable!(ReadWriteRawVec, RawStrategy);
