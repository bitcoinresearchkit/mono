use super::{super::RawStrategy, ReadOnlyRawVec};
use crate::{
    VecIndex, VecValue,
    cache::{Cache, CachePolicy},
    variants::{raw::inner::persisted::RawPersisted, stored_reads::StoredReads},
};

impl<I, T, S, C: CachePolicy> StoredReads<T> for ReadOnlyRawVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    type Persisted<'a>
        = RawPersisted<'a, T, S>
    where
        Self: 'a;

    #[inline(always)]
    fn cache(&self) -> Option<&Cache<T>> {
        C::cache(&self.cache)
    }

    #[inline(always)]
    fn layers(&self) -> (usize, &[T]) {
        (self.base.stored_len(), &[])
    }

    #[inline(always)]
    fn persisted(&self) -> Self::Persisted<'_> {
        RawPersisted::new(&self.base.region)
    }
}

impl_stored_readable!(ReadOnlyRawVec, RawStrategy);
