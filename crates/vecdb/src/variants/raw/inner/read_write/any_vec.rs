use crate::{AnyVec, VecIndex, VecValue, Version, cache::CachePolicy};

use super::{super::RawStrategy, ReadWriteRawVec};

impl<I, T, S, C: CachePolicy> AnyVec for ReadWriteRawVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    #[inline]
    fn version(&self) -> Version {
        self.base.version()
    }

    #[inline]
    fn name(&self) -> &str {
        self.base.name()
    }

    #[inline]
    fn len(&self) -> usize {
        self.base.len()
    }

    crate::any_vec_type_methods!(I, T);

    #[inline]
    fn region_names(&self) -> Vec<String> {
        vec![self.index_to_name()]
    }
}
