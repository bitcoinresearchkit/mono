use crate::{BytesStrategy, VecReader};

pub type BytesVecReader<I, T> = VecReader<I, T, BytesStrategy<T>>;
