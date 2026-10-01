use std::marker::PhantomData;

mod raw;
mod value;

/// Serialization strategy using zerocopy for native byte order access.
///
/// Uses direct memory mapping in native byte order - not portable across endianness.
#[derive(Debug, Clone, Copy)]
pub struct ZeroCopyStrategy<T>(PhantomData<T>);
