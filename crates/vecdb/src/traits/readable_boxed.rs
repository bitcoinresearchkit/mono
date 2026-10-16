use std::{ops::Deref, sync::Arc};

use crate::{AnyVec, ReadableCloneableVec, ReadableVec, TypedVec, VecIndex, VecValue, Version};

use super::chunk_folds;

/// A type-erased read-only reader. Clones share it: readers hold no per-clone state.
#[derive(Clone)]
pub struct ReadableBoxedVec<I, T>(Arc<dyn ReadableCloneableVec<I, T>>)
where
    I: VecIndex,
    T: VecValue;

impl<I, T> ReadableBoxedVec<I, T>
where
    I: VecIndex,
    T: VecValue,
{
    /// Erases `inner`'s read-only form: a stored writer converts to its read-only clone, so no
    /// erased reader sees (or shares) a writer's unsaved values.
    #[inline]
    pub fn new(inner: impl ReadableCloneableVec<I, T> + 'static) -> Self {
        inner.into_read_only_boxed()
    }

    /// Erases a reader that is read-only already.
    #[inline]
    pub(crate) fn from_read_only(inner: impl ReadableCloneableVec<I, T> + 'static) -> Self {
        Self(Arc::new(inner))
    }
}

impl<I, T> Deref for ReadableBoxedVec<I, T>
where
    I: VecIndex,
    T: VecValue,
{
    type Target = dyn ReadableCloneableVec<I, T>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<I, T> AnyVec for ReadableBoxedVec<I, T>
where
    I: VecIndex,
    T: VecValue,
{
    #[inline(always)]
    fn version(&self) -> Version {
        self.0.version()
    }

    #[inline(always)]
    fn name(&self) -> &str {
        self.0.name()
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.0.len()
    }

    #[inline(always)]
    fn is_mutable(&self) -> bool {
        self.0.is_mutable()
    }

    #[inline(always)]
    fn visible_len(&self) -> usize {
        self.0.visible_len()
    }

    #[inline(always)]
    fn index_type_to_string(&self) -> &'static str {
        self.0.index_type_to_string()
    }

    #[inline(always)]
    fn region_names(&self) -> Vec<String> {
        self.0.region_names()
    }

    #[inline(always)]
    fn value_type_to_size_of(&self) -> usize {
        self.0.value_type_to_size_of()
    }

    #[inline(always)]
    fn value_type_to_string(&self) -> &'static str {
        self.0.value_type_to_string()
    }
}

impl<I, T> TypedVec for ReadableBoxedVec<I, T>
where
    I: VecIndex,
    T: VecValue,
{
    type I = I;
    type T = T;
}

impl<I, T> ReadableVec<I, T> for ReadableBoxedVec<I, T>
where
    I: VecIndex,
    T: VecValue,
{
    fn read_cached_into_at(&self, from: usize, to: usize, out: &mut Vec<T>) -> bool {
        self.0.read_cached_into_at(from, to, out)
    }

    #[inline(always)]
    fn cursor_chunk_size(&self) -> usize {
        self.0.cursor_chunk_size()
    }

    #[inline(always)]
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        self.0.read_into_at(from, to, buf);
    }

    #[inline]
    fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
        self.0.for_each_chunk_at(from, to, f);
    }

    #[inline(always)]
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.0.for_each_range_dyn_at(from, to, f);
    }

    #[inline]
    fn fold_range_at<B, F: FnMut(B, T) -> B>(&self, from: usize, to: usize, init: B, f: F) -> B {
        chunk_folds::fold(self, from, to, init, f)
    }

    #[inline]
    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E> {
        chunk_folds::try_fold(self, from, to, init, f)
    }

    #[inline(always)]
    fn collect_one_at(&self, index: usize) -> Option<T> {
        self.0.collect_one_at(index)
    }

    #[inline(always)]
    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        self.0.read_sorted_into_at(indices, out);
    }
}
