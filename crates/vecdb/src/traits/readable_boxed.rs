use std::{convert::Infallible, ops::Deref};

use crate::{AnyVec, ReadableCloneableVec, ReadableVec, TypedVec, VecIndex, VecValue, Version};

pub struct ReadableBoxedVec<I, T>(Box<dyn ReadableCloneableVec<I, T>>)
where
    I: VecIndex,
    T: VecValue;

impl<I, T> ReadableBoxedVec<I, T>
where
    I: VecIndex,
    T: VecValue,
{
    pub fn new(inner: impl ReadableCloneableVec<I, T> + 'static) -> Self {
        Self(Box::new(inner))
    }
}

impl<I, T> Clone for ReadableBoxedVec<I, T>
where
    I: VecIndex,
    T: VecValue,
{
    fn clone(&self) -> Self {
        self.0.read_only_boxed_clone()
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
    #[inline(always)]
    fn has_cache_layer(&self) -> bool {
        self.0.has_cache_layer()
    }

    #[inline(always)]
    fn cursor_chunk_size(&self) -> usize {
        self.0.cursor_chunk_size()
    }

    #[inline(always)]
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        self.0.read_into_at(from, to, buf);
    }

    #[inline(always)]
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.0.for_each_range_dyn_at(from, to, f);
    }

    #[inline]
    fn fold_range_at<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> B {
        self.try_fold_range_at(from, to, init, |acc, value| {
            Ok::<_, Infallible>(f(acc, value))
        })
        .unwrap_or_else(|error| match error {})
    }

    #[inline]
    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> Result<B, E> {
        let to = to.min(self.len());
        let chunk_size = self.cursor_chunk_size().max(1);
        let mut buf = Vec::with_capacity(chunk_size.min(to.saturating_sub(from)));
        let mut acc = init;

        let mut start = from;
        while start < to {
            let end = start.saturating_add(chunk_size).min(to);
            self.read_into_at(start, end, &mut buf);
            for value in buf.drain(..) {
                acc = f(acc, value)?;
            }
            start = end;
        }

        Ok(acc)
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
