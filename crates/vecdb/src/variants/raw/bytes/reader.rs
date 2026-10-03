use std::ops::Deref;

use crate::{BytesStrategy, BytesVecValue, VecIndex, VecReader};

pub struct BytesVecReader<I, T>(VecReader<I, T, BytesStrategy<T>>)
where
    I: VecIndex,
    T: BytesVecValue;

impl<I, T> BytesVecReader<I, T>
where
    I: VecIndex,
    T: BytesVecValue,
{
    #[doc(hidden)]
    pub(crate) fn new(reader: VecReader<I, T, BytesStrategy<T>>) -> Self {
        Self(reader)
    }
}

impl<I, T> From<VecReader<I, T, BytesStrategy<T>>> for BytesVecReader<I, T>
where
    I: VecIndex,
    T: BytesVecValue,
{
    fn from(reader: VecReader<I, T, BytesStrategy<T>>) -> Self {
        Self::new(reader)
    }
}

impl<I, T> Deref for BytesVecReader<I, T>
where
    I: VecIndex,
    T: BytesVecValue,
{
    type Target = VecReader<I, T, BytesStrategy<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
