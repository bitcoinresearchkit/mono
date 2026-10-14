use crate::{AnyVec, VecIndex, VecValue, Version};

use super::LazyVec;

impl<I, T, S1I, S1T> AnyVec for LazyVec<I, T, S1I, S1T>
where
    I: VecIndex,
    T: VecValue,
    S1I: VecIndex,
    S1T: VecValue,
{
    fn version(&self) -> Version {
        self.base_version + self.source.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.source.len()
    }

    /// A view of a mutable source changes with it.
    #[inline]
    fn is_mutable(&self) -> bool {
        self.source.is_mutable()
    }

    crate::any_vec_type_methods!(I, T);
}
