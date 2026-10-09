use crate::{AnyVec, VecIndex, VecValue, Version, short_type_name};

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

    fn index_type_to_string(&self) -> &'static str {
        I::to_string()
    }

    fn len(&self) -> usize {
        self.source.len().min(self.window_starts.len())
    }

    /// A view of a mutable source changes with it.
    #[inline]
    fn is_mutable(&self) -> bool {
        self.source.is_mutable() || self.window_starts.is_mutable()
    }

    #[inline]
    fn value_type_to_size_of(&self) -> usize {
        size_of::<T>()
    }

    #[inline]
    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<T>()
    }

    #[inline]
    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }
}
