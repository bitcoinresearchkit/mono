use crate::{BytesVecValue, ValueStrategy};

use super::BytesStrategy;

impl<T: BytesVecValue> ValueStrategy<T> for BytesStrategy<T> {
    const IS_NATIVE_LAYOUT: bool = T::IS_NATIVE_LAYOUT;

    #[inline(always)]
    fn read(bytes: &[u8]) -> crate::Result<T> {
        T::from_bytes(bytes)
    }

    #[inline(always)]
    fn write_to_vec(value: &T, buf: &mut Vec<u8>) {
        buf.extend_from_slice(value.to_bytes().as_ref());
    }

    #[inline(always)]
    fn write_to_slice(value: &T, dst: &mut [u8]) {
        dst.copy_from_slice(value.to_bytes().as_ref());
    }
}
