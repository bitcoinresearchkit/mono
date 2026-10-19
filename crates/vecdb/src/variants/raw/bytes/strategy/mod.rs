use std::marker::PhantomData;

use crate::BytesVecValue;

pub mod raw;

/// Serialization through [`Bytes`](crate::Bytes).
#[derive(Debug, Clone, Copy)]
pub struct BytesStrategy<T>(PhantomData<T>);

impl_bytes_value_strategy!(BytesStrategy, BytesVecValue);
