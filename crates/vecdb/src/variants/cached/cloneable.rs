use std::sync::Arc;

use crate::{AnyVec, ReadOnlyClone, ReadableVec, StoredVec, TypedVec, VecIndex, VecValue};

use super::CachedVec;

pub trait CachedReadableVec<I, T>: AnyVec + Send + Sync
where
    I: VecIndex,
    T: VecValue,
{
    fn snapshot(&self) -> Arc<Vec<T>>;
    fn invalidate(&self);
    fn cached_boxed_clone(&self) -> CachedBoxedVec<I, T>;
}

pub type CachedBoxedVec<I, T> = Box<dyn CachedReadableVec<I, T>>;

impl<I, T> Clone for CachedBoxedVec<I, T>
where
    I: VecIndex,
    T: VecValue,
{
    fn clone(&self) -> Self {
        self.cached_boxed_clone()
    }
}

impl<I, T, V> CachedReadableVec<I, T> for CachedVec<V>
where
    I: VecIndex,
    T: VecValue,
    V: TypedVec<I = I, T = T> + ReadableVec<I, T> + Clone + Send + Sync + 'static,
{
    fn snapshot(&self) -> Arc<Vec<T>> {
        CachedVec::snapshot(self)
    }

    fn invalidate(&self) {
        CachedVec::invalidate(self);
    }

    fn cached_boxed_clone(&self) -> CachedBoxedVec<I, T> {
        Box::new(self.clone())
    }
}

impl<V> CachedVec<V>
where
    V: StoredVec,
    V::ReadOnly:
        TypedVec<I = V::I, T = V::T> + ReadableVec<V::I, V::T> + Clone + Send + Sync + 'static,
{
    pub fn read_only_cached_boxed_clone(&self) -> CachedBoxedVec<V::I, V::T> {
        Box::new(self.read_only_clone())
    }
}
