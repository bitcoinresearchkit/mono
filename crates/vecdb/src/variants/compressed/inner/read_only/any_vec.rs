use crate::{AnyVec, VecIndex, VecValue, Version, cache::CachePolicy, vec_region_name};

use super::{super::CompressionStrategy, ReadOnlyCompressedVec};

impl<I, T, S, C: CachePolicy> AnyVec for ReadOnlyCompressedVec<I, T, S, C>
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
        &self.base.name
    }

    /// Append-only: rows below its length never change.
    #[inline]
    fn is_mutable(&self) -> bool {
        false
    }

    #[inline]
    fn len(&self) -> usize {
        self.base.stored_len()
    }

    crate::any_vec_type_methods!(I, T);

    #[inline]
    fn region_names(&self) -> Vec<String> {
        let base = vec_region_name(&self.base.name, I::to_string());
        let pages = format!("{base}_pages");
        vec![base, pages]
    }
}
