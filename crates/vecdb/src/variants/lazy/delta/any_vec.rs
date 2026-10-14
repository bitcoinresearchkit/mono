use crate::{AnyVec, VecIndex, VecValue, Version};

use super::{DeltaOp, LazyDeltaVec};

impl<I, S, T, Op> AnyVec for LazyDeltaVec<I, S, T, Op>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
    Op: DeltaOp<S, T>,
{
    fn version(&self) -> Version {
        self.base_version + self.source.version() + self.window_starts.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.source.len().min(self.window_starts.len())
    }

    /// A view of a mutable source changes with it.
    #[inline]
    fn is_mutable(&self) -> bool {
        self.source.is_mutable() || self.window_starts.is_mutable()
    }

    crate::any_vec_type_methods!(I, T);
}
