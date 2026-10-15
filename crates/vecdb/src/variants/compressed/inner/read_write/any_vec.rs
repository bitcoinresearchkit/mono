use crate::{AnyVec, VecIndex, VecValue, Version, cache::CachePolicy};

use super::{super::CompressionStrategy, ReadWriteCompressedVec};

impl<I, T, S, C: CachePolicy> AnyVec for ReadWriteCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    #[inline]
    fn version(&self) -> Version {
        self.base.version()
    }

    #[inline]
    fn name(&self) -> &str {
        self.base.name()
    }

    /// Append-only: rows below its length never change (a `MutableVec` wrapping it says otherwise).
    #[inline]
    fn is_mutable(&self) -> bool {
        false
    }

    #[inline]
    fn len(&self) -> usize {
        self.base.len()
    }

    crate::any_vec_type_methods!(I, T);

    #[inline]
    fn region_names(&self) -> Vec<String> {
        vec![self.base.index_to_name(), self.pages_region_name()]
    }
}
