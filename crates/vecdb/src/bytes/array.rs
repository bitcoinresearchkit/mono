use crate::Error;

use super::Bytes;

// SAFETY: a byte array is its own byte image.
unsafe impl<const N: usize> Bytes for [u8; N] {
    type Array = [u8; N];
    const IS_NATIVE_LAYOUT: bool = true;

    #[inline]
    fn to_bytes(&self) -> Self::Array {
        *self
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        bytes.try_into().map_err(|_| Error::WrongLength {
            expected: N,
            received: bytes.len(),
        })
    }
}
