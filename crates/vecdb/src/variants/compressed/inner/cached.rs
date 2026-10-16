use crate::{Result, VecIndex, VecValue, cache::CachePolicy};

use super::{CompressionStrategy, ReadWriteCompressedVec};

impl<I, T, S, C> ReadWriteCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
    C: CachePolicy,
{
    pub(super) fn with_cache_update<R>(
        &mut self,
        from: usize,
        write: impl FnOnce(&mut Self) -> Result<R>,
    ) -> Result<R> {
        C::update(self.cache.clone(), from, || write(self))
    }
}
