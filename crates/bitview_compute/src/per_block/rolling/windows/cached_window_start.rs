use std::sync::Arc;

use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{
    AnyVec, CachedBoxedVec, CachedReadableVec, CachedVec, ReadOnlyClone, ReadableVec, TypedVec,
};

use super::LazyWindowStartVec;

/// A pinned, process-lifetime cache for a heavily reused window-start vector.
#[derive(Clone, Traversable)]
#[traversable(transparent)]
pub struct CachedWindowStartVec(CachedVec<LazyWindowStartVec>);

impl CachedWindowStartVec {
    pub fn new(inner: LazyWindowStartVec) -> Self {
        Self(CachedVec::wrap(inner))
    }

    pub fn lazy(&self) -> &LazyWindowStartVec {
        &self.0.inner
    }

    pub fn version(&self) -> Version {
        self.0.version()
    }

    pub fn read_only_cached_boxed_clone(&self) -> CachedBoxedVec<Height, Height> {
        self.0.cached_boxed_clone()
    }

    pub fn snapshot(&self) -> Arc<Vec<Height>> {
        self.0.snapshot()
    }

    pub fn invalidate(&self) {
        self.0.invalidate();
    }
}

impl AnyVec for CachedWindowStartVec {
    fn version(&self) -> Version {
        self.0.version()
    }

    fn name(&self) -> &str {
        self.0.name()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn index_type_to_string(&self) -> &'static str {
        self.0.index_type_to_string()
    }

    fn region_names(&self) -> Vec<String> {
        self.0.region_names()
    }

    fn value_type_to_size_of(&self) -> usize {
        self.0.value_type_to_size_of()
    }

    fn value_type_to_string(&self) -> &'static str {
        self.0.value_type_to_string()
    }
}

impl TypedVec for CachedWindowStartVec {
    type I = Height;
    type T = Height;
}

impl ReadableVec<Height, Height> for CachedWindowStartVec {
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<Height>) {
        self.0.read_into_at(from, to, buf);
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(Height)) {
        self.0.for_each_range_dyn_at(from, to, each);
    }

    fn fold_range_at<B, F: FnMut(B, Height) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> B {
        self.0.fold_range_at(from, to, init, fold)
    }

    fn try_fold_range_at<B, E, F: FnMut(B, Height) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> Result<B, E> {
        self.0.try_fold_range_at(from, to, init, fold)
    }

    fn collect_one_at(&self, index: usize) -> Option<Height> {
        self.0.collect_one_at(index)
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<Height>) {
        self.0.read_sorted_into_at(indices, out);
    }
}

impl ReadOnlyClone for CachedWindowStartVec {
    type ReadOnly = Self;

    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}
