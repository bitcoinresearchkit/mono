use std::slice;

use super::BytesStrategy;
use crate::{BytesVecValue, Format, ValueStrategy, variants::raw::RawStrategy};

impl<T: BytesVecValue> RawStrategy<T> for BytesStrategy<T> {
    const FORMAT: Format = Format::Bytes;

    #[inline(always)]
    unsafe fn read_from_ptr(ptr: *const u8, byte_offset: usize) -> T {
        unsafe {
            if T::IS_NATIVE_LAYOUT {
                (ptr.add(byte_offset) as *const T).read_unaligned()
            } else {
                let slice = slice::from_raw_parts(ptr.add(byte_offset), size_of::<T>());
                // Stored bytes come from `to_bytes`, but a rejected value must not become UB.
                <BytesStrategy<T> as ValueStrategy<T>>::read(slice).expect("invalid stored value")
            }
        }
    }
}
