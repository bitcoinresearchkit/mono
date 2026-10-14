use super::ZeroCopyStrategy;
use crate::{Error, Result, ValueStrategy, ZeroCopyVecValue};

// SAFETY: zerocopy's `FromBytes + IntoBytes` bounds make every value a plain, padding-free byte
// image valid for every bit pattern.
unsafe impl<T: ZeroCopyVecValue> ValueStrategy<T> for ZeroCopyStrategy<T> {
    const IS_NATIVE_LAYOUT: bool = true;

    #[inline(always)]
    fn read(bytes: &[u8]) -> Result<T> {
        T::read_from_prefix(bytes)
            .map(|(v, _)| v)
            .map_err(|_| Error::ZeroCopyError)
    }

    #[inline(always)]
    fn write_to_vec(value: &T, buf: &mut Vec<u8>) {
        buf.extend_from_slice(value.as_bytes());
    }

    #[inline(always)]
    fn write_to_slice(value: &T, dst: &mut [u8]) {
        dst.copy_from_slice(value.as_bytes());
    }
}
