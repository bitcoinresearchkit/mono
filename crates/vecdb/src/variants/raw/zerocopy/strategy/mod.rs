use std::marker::PhantomData;

mod raw;
mod value;

/// Native-layout serialization through zerocopy.
#[derive(Debug, Clone, Copy)]
pub struct ZeroCopyStrategy<T>(PhantomData<T>);
