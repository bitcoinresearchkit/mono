use std::ops::Deref;

use crate::{ReadableCloneableVec, VecIndex, VecValue};

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
