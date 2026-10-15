use crate::{BytesStrategy, VecReader};

/// Random-access reader over a [`BytesVec`](crate::BytesVec).
pub type BytesVecReader<I, T> = VecReader<I, T, BytesStrategy<T>>;
