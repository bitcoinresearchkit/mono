use std::{ops::Deref, sync::Arc};

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

pub struct CachedBoxedVec<I, T>(Box<dyn CachedReadableVec<I, T>>)
where
    I: VecIndex,
    T: VecValue;

impl<I, T> CachedBoxedVec<I, T>
where
    I: VecIndex,
    T: VecValue,
{
    pub fn new(inner: impl CachedReadableVec<I, T> + 'static) -> Self {
        Self(Box::new(inner))
    }
}

impl<I, T> Clone for CachedBoxedVec<I, T>
where
    I: VecIndex,
    T: VecValue,
{
    fn clone(&self) -> Self {
        self.0.cached_boxed_clone()
    }
}

impl<I, T> Deref for CachedBoxedVec<I, T>
where
    I: VecIndex,
    T: VecValue,
{
    type Target = dyn CachedReadableVec<I, T>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
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
        CachedBoxedVec::new(self.clone())
    }
}

impl<V> CachedVec<V>
where
    V: StoredVec,
    V::ReadOnly:
        TypedVec<I = V::I, T = V::T> + ReadableVec<V::I, V::T> + Clone + Send + Sync + 'static,
{
    pub fn read_only_cached_boxed_clone(&self) -> CachedBoxedVec<V::I, V::T> {
        CachedBoxedVec::new(self.read_only_clone())
    }
}
