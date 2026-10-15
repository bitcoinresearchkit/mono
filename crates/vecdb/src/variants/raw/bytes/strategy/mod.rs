use std::marker::PhantomData;

use crate::BytesVecValue;

pub mod raw;

/// Serialization strategy using the Bytes trait with portable byte order.
///
/// Implements little-endian serialization for cross-platform compatibility.
#[derive(Debug, Clone, Copy)]
pub struct BytesStrategy<T>(PhantomData<T>);

impl_bytes_value_strategy!(BytesStrategy, BytesVecValue);
