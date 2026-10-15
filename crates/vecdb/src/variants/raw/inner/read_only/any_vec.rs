use crate::{AnyVec, VecIndex, VecValue, Version, cache::CachePolicy, vec_region_name};

use super::{super::RawStrategy, ReadOnlyRawVec};

impl<I, T, S, C: CachePolicy> AnyVec for ReadOnlyRawVec<I, T, S, C>
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
        vec![vec_region_name(self.base.name(), I::to_string())]
    }
}
